#![allow(dead_code, unused_variables, unused_imports)]

use fantoccini::{Client, ClientBuilder, Locator};
use panda::Panda;
use std::process::Command;
use std::{error::Error, process::Stdio, time::Duration};
use tokio::time::sleep;

mod panda;

type Result<T> = core::result::Result<T, Box<dyn Error>>;

const EMAIL: &str = "psoreto@outlook.com";
const PASSWORD: &str = "Ab@12345";
const WEBDRIVER_PATH: &str = "C:\\Users\\paulo\\dev\\webdrivers\\geckodriver.exe";

#[derive(Debug)]
struct NovelInfo {
    title: String,
    current_chapter: f32,
    last_available_chapter: f32,
    available_chapters_count: usize,
    current_chapter_url: String,
}

async fn skip_popup(c: &Client) -> Result<()> {
    c.wait()
        .for_element(Locator::XPath("//div[@class='ui-dialog-icon']"))
        .await?;

    c.find(Locator::XPath("//div[@class='ui-dialog-icon']"))
        .await?
        .find(Locator::XPath("//button[@class='btn btn-default']"))
        .await?
        .click()
        .await?;

    Ok(())
}

async fn sign_in(c: &Client) -> Result<()> {
    c.find(Locator::XPath("//input[@name='email']"))
        .await?
        .send_keys(EMAIL)
        .await?;

    c.find(Locator::XPath("//input[@name='password']"))
        .await?
        .send_keys(PASSWORD)
        .await?;

    sleep(Duration::from_secs(1)).await;

    c.find(Locator::XPath("//button[@class='btn btn-submit']"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_secs(1)).await;

    Ok(())
}

async fn get_library_info(c: &Client) -> Result<Vec<NovelInfo>> {
    c.wait().for_element(Locator::Css(".library-list")).await?;
    sleep(Duration::from_secs(5)).await;

    let mut novels: Vec<NovelInfo> = vec![];
    let lis = c
        .find_all(Locator::XPath("//div[@class='library-list']//ul[1]//li"))
        .await?;

    for li in lis {
        let title = li.find(Locator::Css("h4")).await?.text().await?;
        let nums: Vec<f32> = li
            .find(Locator::Css("h6>em"))
            .await?
            .text()
            .await?
            .split('/')
            .map(|s| s.trim())
            .map(|s| s.parse().unwrap())
            .collect();
        let current_chapter_url = li
            .find(Locator::Css("a"))
            .await?
            .prop("href")
            .await?
            .unwrap();

        let current_chapter = nums.get(0).unwrap().clone();
        let last_available_chapter = nums.get(1).unwrap().clone();
        let available_chapters_count = (last_available_chapter - current_chapter) as usize;

        let novel = NovelInfo {
            title,
            current_chapter,
            last_available_chapter,
            available_chapters_count,
            current_chapter_url,
        };

        novels.push(novel);
    }

    Ok(novels)
}

async fn run() -> Result<()> {
    let panda = Panda::new(Some(WEBDRIVER_PATH), None).await?;

    // let week_new_novels = panda.week_new_novels().await?;
    // let week_hot_novels = panda.week_hot_novels().await?;

    let term = "my vampire wife";
    println!("Searching for '{}'...", term);
    let result = panda.search(term).await?;
    println!("Found {} results.", result.len());
    let my_daughter_is_a_vampire = result.get(2).unwrap();
    println!("{:?}", my_daughter_is_a_vampire);
    // for info in result {
    //     println!("{:?}", info);
    //     println!("");
    // }

    Ok(())
    // let c = ClientBuilder::native()
    //     .connect("http://localhost:4444")
    //     .await?;

    // c.goto("https://www.panda-novel.com/email-login").await?;

    // skip_popup(&c).await?;
    // sign_in(&c).await?;

    // c.goto("https://www.panda-novel.com/library").await?;

    // let _novels = get_library_info(&c).await?;

    // iterar todas as novels
    // ccoletar o conteudo de cada capitulo ate o ultimo
    // salvar em um .md contendo todos os capitulo separados por alguma coisa

    // let _ = c.wait().forever();

    // Ok(())
}

fn main() {
    // let mut cmd = Command::new("C:\\Users\\paulo\\dev\\webdrivers\\geckodriver.exe")
    //     .stdout(Stdio::null())
    //     .spawn()
    //     .unwrap();

    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());

    if result.is_err() {
        println!("Err: '{}'", result.unwrap_err().to_string());
    }

    // cmd.kill().unwrap();
}
