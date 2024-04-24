

use std::process::exit;
/// @brief findin cerca una stringa dentro un file di test
/// @author Cristiano Chieppa
/// @date Aprile 2024

use std::{fs, io};
use std::env;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::thread::JoinHandle;


const MAX_FILE_SIZE:u64 = 5000;

fn main() -> io::Result<()> {
    let num_of_threads = thread::available_parallelism()?.get();
    println!("num of threads = {}", num_of_threads);

    let args :Vec<String> = env::args().skip(1).collect();

    if args.len() != 2
    {
        eprintln!("numero di argomenti insufficiente");
        exit(0);
    }

    // debug
    println!("Current folder {}", env::current_dir().unwrap().display());

    // apriamo il file
    //let text = fs::read_to_string(&args[0])?;
    let refe_text =  Arc::new(fs::read_to_string(&args[0])?);

    // usiamo il parallel per leggere quante righe ci sono
    // let num_of_lines = text.as_str().par_chars().filter(|c| *c == '\n').count()+1;

    let file_size = fs::metadata(&args[0])?.len();

    let threads:u64 = match file_size >= MAX_FILE_SIZE  {
        true => num_of_threads as u64,
        false => 1
    };

    println!("File size = {}", file_size);
    let offset = file_size / threads;

    let bytes = refe_text.as_str().as_bytes();
    let mut ranges = Vec::<(u64,u64)>::new();

    let mut inf = 0;
    for i in 0..threads
    {
        //let inf = i * offset;
        let mut sup = i*offset+offset;

        // controlliamo che sup non sia in mezzo a una parola
        // cercando lo spazio o a capo successivo
        loop
        {
            if sup == file_size
            {
                break;
            }

            match bytes[sup as usize-1] {
                b' ' => break,
                b'\n' => break,
                _ => sup += 1,
            };

            //println!("\tsono in mezzo alla parola: {}, {}", sup, bytes[sup as usize-1] as char );
        }

        ranges.push((inf, sup));
        inf = sup;
    }

    for (inf, sup) in &ranges
    {
        println!("[{}..{}[", inf, sup);
    }

    // per ogni thread cerchiamo la parola
    let (tx, rx) :(Sender<bool>, Receiver<bool>) =  mpsc::channel();
    let mut handles = Vec::<JoinHandle<_>>::new();

    for index in 0..threads
    {
        let (inf, sup) = ranges[index as usize];
        let thread_tx = tx.clone();

        let copy =  Arc::clone(&refe_text); // text.clone();
        let to_find = args[1].clone();
        let handle = thread::spawn( move ||
        {
            let slice = &copy[inf as usize..sup as usize];
            match slice.contains(&to_find)
            {
                true => thread_tx.send(true).unwrap(),
                false => thread_tx.send(false).unwrap(),
            }
        });
        handles.push(handle);
    }

    let mut ids = Vec::with_capacity(num_of_threads as usize);
    for _ in 0..threads {
        ids.push(rx.recv());
    }

    match ids.contains(&Ok(true))
    {
        true => println!("Trovato"),
        false => println!("Non trovato"),
    }

    for handle in handles
    {
        let _ = handle.join().expect("Thread inpanicato");
    }
    Ok(())
}
