use std::sync::mpsc::Sender;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_str;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Semaphore;
#[derive(Serialize, Deserialize, Debug)]
struct Story {
    by: String,
    descendants: i32,
    id: i32,
    kids: Vec<i32>,
    score: i32,
    time: i32,
    title: String,
    r#type: String,
    url: String,
}

impl fmt::Display for Story {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //this doesn't include the kids because you can't 
        write!(
            f,
            "({},{},{},{},{},{},{},{})",
            self.by,
            self.descendants,
            self.id,
            self.score,
            self.time,
            self.title,
            self.r#type,
            self.url
        )
    }
}
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let channel_buffer: i32;
    if let Ok(resp) = get_max_item().await {
        channel_buffer = resp.parse::<i32>().unwrap();
        println!("{}", resp);
    } else {
        panic!("wrong link")
    }
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Story>(channel_buffer as usize);

    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(100));
        let mut join_handles = Vec::new();
        //for i in 1..channel_buffer as i32 {
        for i in 1..1000 as i32 {
            //println!("{}", i);
            let curr_tx = tx.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            join_handles.push(tokio::spawn(async move {
                if let Some(ok) = get_story(i).await {
                    if let Err(_) = curr_tx.send(ok).await {
                        format!("error sending {} through channel", i);
                    };
                }
                drop(permit);
            }));
        }
        for handle in join_handles {
            handle.await.unwrap();
        }
    });

    while let Some(message) = rx.recv().await {
        println!("{:?}", message.to_string());
    }
    Ok(())
}

async fn get_max_item() -> Result<String, reqwest::Error> {
    match reqwest::get("https://hacker-news.firebaseio.com/v0/maxitem.json?print=pretty")
        .await?
        .text()
        .await
    {
        Ok(response) => {
            //println!("{}", response.trim());
            Ok(response.trim().to_owned())
        }
        Err(e) => Err(e),
    }
}
async fn get_story(i: i32) -> Option<Story> {
    let search_type = "item";
    let url = format!(
        "https://hacker-news.firebaseio.com/v0/{}/{}.json?print=pretty",
        search_type, i
    );
    let response = reqwest::get(url).await;
    if let Ok(resp) = response {
        let buff = String::from(resp.text().await.unwrap());
        //Ok(from_str::<Story>(&buff))
        match serde_json::from_str::<Story>(&buff) {
            Ok(story) => Some(story),
            Err(_) => None,
        }
    } else {
        None
    }
}
