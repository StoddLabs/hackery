use std::sync::mpsc::Sender;

use serde::Deserialize;
use serde::Serialize;
use serde_json::from_str;

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

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut channel_buffer: i32 = 1;
    if let Ok(resp) = get_max_item().await {
        channel_buffer = resp.parse::<i32>().unwrap();
        println!("{}", resp);
    } else {
        panic!("wrong link")
    }
    let (mut tx, mut rx) = tokio::sync::mpsc::channel::<Story>(channel_buffer as usize);

    //for i in 1..channel_buffer as i32 {
    let current = 0;
    // tokio::spawn(async move {
    //     for i in 1..1000 as i32 {

    //         let curr_tx = tx.clone();
    //         tokio::spawn(async move {
    //             if let Some(ok) = get_story(i).await {
    //                 //println!("{}", i);
    //                 curr_tx.send(ok).await;
    //             }
    //         });
    //     }
    // });
    tokio::spawn(async move {
        for i in 1..channel_buffer as i32 {
            let curr_tx = tx.clone();
            let fut = tokio::spawn(async move {
                if let Some(ok) = get_story(i).await {
                    curr_tx.send(ok).await;
                }
            });
            println!("{}", i);
            if i % 500000 == 0 {
                println!("awaiting");
                fut.await;
            }
        }
    });
    while let Some(message) = rx.recv().await {
        //println!("GOT = {:?}", message);
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
        let mut buff = String::from(resp.text().await.unwrap());
        //Ok(from_str::<Story>(&buff))
        match serde_json::from_str::<Story>(&buff) {
            Ok(story) => Some(story),
            Err(_) => None,
        }
    } else {
        None
    }
}
