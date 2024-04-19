use std::fs;
use std::io::{self, Write};
use colored::Colorize;
use reqwest;
use tokio::task;
use tokio::sync::Semaphore;
use std::sync::Arc;

const MAX_CONCURRENT_TASKS: usize = 200; // Adjust the number to your preference
const PRINT_INTERVAL: usize = 100; // Adjust the interval to print the checked count

async fn get_request(word: String, url_str: String, counter: &Arc<std::sync::atomic::AtomicUsize>, total_words: usize) {
    let url = format!("{}/{}", url_str, word);
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().as_u16() == 200 && response.status().as_str().contains("200") {
                    println!(
                        "Status: {} || URL: {}", 
                        format!("{}", response.status()).green(),
                        url
                    );
            }
        }
        Err(err) => eprintln!("Error: {}", err),
    }
    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let counter_check = format!("{}", counter.load(std::sync::atomic::Ordering::Relaxed));
    print!("\r Counting Words: {} ||  ", counter_check.yellow());
    io::stdout().flush().unwrap();
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let banner = r#"
::::::::      ::: ::::::::::: ::::::::  :::    :::      :::::::::   ::::::::  :::    ::: ::::::::::: :::::::::: ::::::::  
:+:    :+:   :+: :+:   :+:    :+:    :+: :+:    :+:      :+:    :+: :+:    :+: :+:    :+:     :+:     :+:       :+:    :+: 
+:+         +:+   +:+  +:+    +:+        +:+    +:+      +:+    +:+ +:+    +:+ +:+    +:+     +:+     +:+       +:+        
+#+        +#++:++#++: +#+    +#+        +#+    +#+      +#+    +#+ +:+    +#+ +#+    +#+     +:+     +#++:++#  +#+ +#+#+# 
+#+        +#+     +#+ +#+    +#+        +#+    +#+      +#+    +#+ +#+    +#+ +#+    +#+     +:+     +#+       +#+    +#+ 
#+#    #+# #+#     #+# #+#    #+#    #+# #+#    #+#      #+#    #+# #+#    #+# #+#    #+#     #+#     #+#       #+#    #+# 
 ########  ###     ### ###     ########  ###    ###      ###    ###  ########   ########      ###     ########## ########  

CatchRoutes - A Wordlist BruteForcer Tool.
Replace Your Custom Wordlist with your Default Wordlist with the same name.

URL example: - https://example.com, http://example.com
"#;

    print!("{}\n", banner.bright_cyan());
    let message = "Please Enter Your URL";
    print!("    {} : => ", message.green());
    io::stdout().flush()?;
    let mut url_str = String::new();
    io::stdin().read_line(&mut url_str)?;

    let url_str = url_str.trim().to_string();

    println!("* Process Started * \n");

    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));

    let wordlist = fs::read_to_string("wordlist.txt")?;
    let total_words = wordlist.lines().count();

    let mut last_counter = 0;
    let tasks: Vec<_> = wordlist.lines()
        .enumerate()
        .map(|(index, line)| {
            let semaphore = Arc::clone(&semaphore);
            let url_str_clone = url_str.clone(); // Clone url_str for each task
            let word = line.trim().to_string();
            let counter = Arc::clone(&counter);
            task::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Failed to acquire semaphore permit");
                get_request(word, url_str_clone, &counter, total_words).await;
                if index % PRINT_INTERVAL == 0 {
                    if counter.load(std::sync::atomic::Ordering::Relaxed) > last_counter {
                        let countercheck = format!("{}", counter.load(std::sync::atomic::Ordering::Relaxed));
                        print!("\r{}", countercheck.bright_yellow());
                        io::stdout().flush().unwrap();
                    }
                }
            })
        })
        .collect();

    // Wait for all tasks to complete concurrently
    futures::future::join_all(tasks).await;

    println!("\nTotal words checked: {}", counter.load(std::sync::atomic::Ordering::Relaxed));

    Ok(())
}