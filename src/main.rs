use std::fs;
use std::io::{self, Write};
use colored::Colorize;
use reqwest;
use tokio::task;
use tokio::sync::Semaphore;
use std::sync::Arc;

const MAX_CONCURRENT_TASKS: usize = 100; // Adjust the number to your preference

async fn get_request(word: String, url_str: String) {
    let url = format!("{}/{}", url_str, word);
    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().as_u16() == 200 {
                println!(
                    "Status: {} for URL: {}", 
                    format!("{}", response.status()).green(),
                    url
                );
            } 
        }
        Err(err) => eprintln!("Error: {}", err),
    }
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
Replace Your Custom Wordlist with your Default Wordlist with same name.

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

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));

    let tasks: Vec<_> = fs::read_to_string("wordlist.txt")?
        .lines()
        .map(|line| {
            let semaphore = Arc::clone(&semaphore);
            let url_str_clone = url_str.clone(); // Clone url_str for each task
            let word = line.trim().to_string();
            task::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Failed to acquire semaphore permit");
                get_request(word, url_str_clone).await;
            })
        })
        .collect();

    // Wait for all tasks to complete
    futures::future::join_all(tasks).await;

    Ok(())
}