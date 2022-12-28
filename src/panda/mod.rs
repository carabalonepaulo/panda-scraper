#![allow(dead_code, unused_variables, unused_imports)]

use core::fmt;
use std::{
    fmt::Formatter,
    process::{Child, Command, Stdio},
    rc::Rc,
    time::Duration,
};

use chrono::NaiveDateTime;
use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    elements::Element,
    key::Key,
    Client, ClientBuilder, Locator,
};
use regex::Regex;
use std::error::Error;
use tokio::{task, time::sleep};
use urlencoding::encode;

mod session;
mod tag;

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;

pub const DEFAULT_WEBDRIVER_URL: &str = "http://localhost:4444";

#[derive(Debug)]
pub enum TranslationKind {
    Manual,
    Machine,
}

#[derive(Debug)]
pub enum Status {
    Ongoing,
    Completed,
}

#[derive(Debug)]
pub enum Country {
    China,
    Korea,
    Japan,
}

#[derive(Debug)]
pub enum Gender {
    Male,
    Female,
}

// #[derive(Debug)]
// pub struct Novel {
//     title: String,
//     author: String,
//     country: Option<Country>,
//     tags: Option<Vec<String>>,
//     chapters_count: usize,
//     views: u64,
//     status: Option<Status>,
//     cover_url: String,
// }

pub struct Novel {
    browser: Rc<Client>,

    url: String,
    cover_url: String,

    title: String,
    tags: Vec<String>,
    views: Option<usize>,

    last_update: Option<NaiveDateTime>,
}

impl Novel {
    pub async fn get_chapter(&self, chapter_num: usize) -> String {
        String::new()
    }

    pub async fn load_info(&mut self) {}
}

impl fmt::Debug for Novel {
    fn fmt(&self, f: &mut fmt::Formatter) -> core::result::Result<(), fmt::Error> {
        write!(
            f,
            "Novel {{\n  url: {}\n  cover_url: {}\n  title: {}\n  tags: {:?}\n  views: {}\n  last_update: {}\n}}",
            self.url, self.cover_url, self.title, self.tags, self.views.unwrap_or(0), self.last_update.unwrap_or(NaiveDateTime::default())
        )
    }
}

#[derive(Debug)]
pub struct Panda {
    child_process: Child,
    browser: Rc<Client>,
}

impl Drop for Panda {
    fn drop(&mut self) {
        self.child_process.kill().unwrap();
    }
}

impl Panda {
    pub async fn new(webdriver_path: Option<&str>, webdriver_url: Option<&str>) -> Result<Self> {
        // "C:\\Users\\paulo\\dev\\webdrivers\\geckodriver.exe"

        let child_process = Command::new(webdriver_path.unwrap_or("geckodriver"))
            .stdout(Stdio::null())
            .spawn()
            .unwrap();

        let browser = ClientBuilder::native()
            .connect(webdriver_url.unwrap_or(DEFAULT_WEBDRIVER_URL))
            .await?;

        Ok(Panda {
            child_process,
            browser: Rc::new(browser),
        })
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<session::Session> {
        Ok(session::Session::new(self.browser.clone()).await?)
    }

    pub async fn week_new_novels(&self) -> Result<Vec<Novel>> {
        let section = self.get_section(0).await?.unwrap();
        let title = section
            .find(Locator::Css(".section-container>.section-title>span"))
            .await?
            .text()
            .await?;

        let lis = section.find_all(Locator::Css(".novel-li")).await?;
        let mut novels: Vec<Novel> = vec![];

        sleep(Duration::from_secs(2)).await;

        for li in lis {
            let url = li
                .find(Locator::Css("a"))
                .await?
                .prop("href")
                .await?
                .unwrap();

            let cover_url = {
                let style = li
                    .find(Locator::Css(".novel-cover>.novel-img"))
                    .await?
                    .attr("style")
                    .await?
                    .unwrap();
                let cover_regex = Regex::new(r#"background-image: url\("(.*)"\);"#).unwrap();
                let captures = cover_regex.captures(style.as_str()).unwrap();
                captures.get(1).unwrap().as_str().to_string()
            };

            let title = li
                .find(Locator::Css("div:nth-child(2) > h4:nth-child(1)"))
                .await?
                .text()
                .await?;

            let main_tag = li
                .find(Locator::Css(".novel-desc>.n-labels>em"))
                .await?
                .text()
                .await?;

            novels.push(Novel {
                browser: self.browser.clone(),

                url,
                cover_url,

                title,
                tags: vec![main_tag],
                views: None,

                last_update: None,
            })
        }

        Ok(novels)
    }

    pub async fn week_hot_novels(&self) -> Result<Vec<Novel>> {
        let section = self.get_section(1).await?.unwrap();
        let title = section
            .find(Locator::Css(".section-container>.section-title>span"))
            .await?
            .text()
            .await?;

        let lis = section
            .find_all(Locator::Css(".ss-right>div>ul>li"))
            .await?;
        let mut novels: Vec<Novel> = vec![];

        self.scroll_down(12).await?;
        sleep(Duration::from_secs(2)).await;

        for li in lis {
            let url = li
                .find(Locator::Css("a"))
                .await?
                .prop("href")
                .await?
                .unwrap();

            let cover_url = {
                let style = li
                    .find(Locator::Css(".novel-cover>.novel-img"))
                    .await?
                    .attr("style")
                    .await?
                    .unwrap();
                let cover_regex = Regex::new(r#"background-image: url\("(.*)"\);"#).unwrap();
                let captures = cover_regex.captures(style.as_str()).unwrap();
                captures.get(1).unwrap().as_str().to_string()
            };

            let title = li
                .find(Locator::Css("div:nth-child(2) > h4:nth-child(1)"))
                .await?
                .text()
                .await?;

            let views = li
                .find(Locator::Css(".novel-desc>.n-labels>label>em"))
                .await?
                .text()
                .await?;

            novels.push(Novel {
                browser: self.browser.clone(),

                url,
                cover_url,

                title,
                tags: vec![],
                views: match views.parse::<usize>() {
                    Ok(views) => Some(views),
                    Err(_) => None,
                },

                last_update: None,
            })
        }

        Ok(novels)
    }

    pub async fn recommended_novels(&self) {}

    pub async fn latest_updates(&self) {}

    // pub async fn get_novel_info_from_url(&self, url: &str) -> Novel {}

    pub async fn get_chapter_from_novel(&self, novel: &Novel) {}

    pub async fn search(&self, term: &str) -> Result<Vec<Novel>> {
        let mut url = String::from("https://www.panda-novel.com/search/");
        url.push_str(&encode(term).to_string());

        self.browser.goto(url.as_str()).await?;
        self.close_annoying_popup().await?;

        let lis = self.browser.find_all(Locator::Css(".novel-li")).await?;
        let mut novels: Vec<Novel> = vec![];

        for li in lis {
            let cover_url = {
                let style = li
                    .find(Locator::Css(".novel-cover>i"))
                    .await?
                    .attr("style")
                    .await?
                    .unwrap();
                let cover_regex = Regex::new(r#"background-image: url\("(.*)"\);"#).unwrap();
                let captures = cover_regex.captures(style.as_str()).unwrap();
                captures.get(1).unwrap().as_str().to_string()
            };

            let url = li
                .find(Locator::Css("a"))
                .await?
                .prop("href")
                .await?
                .unwrap();

            let title = li
                .find(Locator::Css(".novel-desc>h4"))
                .await?
                .text()
                .await?;

            let author = li
                .find(Locator::Css(".novel-desc>h5"))
                .await?
                .text()
                .await?;

            let synopsis = li.find(Locator::Css(".novel-desc>p")).await?.text().await?;

            let chapters = li
                .find(Locator::Css(".novel-desc>.n-attrs>label>em"))
                .await?
                .text()
                .await?;

            let views = li
                .find(Locator::XPath(
                    "//div[@class='novel-desc']//h6[1]//label[1]//em[1]",
                ))
                .await?
                .text()
                .await?
                .parse::<usize>()?;

            let last_update_raw = li
                .find(Locator::XPath(
                    "//div[@class='novel-desc']//h6[1]//label[2]//em[1]",
                ))
                .await?
                .attr("data-listime")
                .await?
                .unwrap();
            let last_update = NaiveDateTime::parse_from_str(&last_update_raw, "%Y-%m-%d %H:%M:%S")?;

            novels.push(Novel {
                browser: self.browser.clone(),

                url,
                cover_url,

                title,
                tags: vec![],
                views: None,

                last_update: None,
            });
        }
        Ok(novels)
    }

    pub async fn browse(
        &self,
        tags: Vec<&str>,
        translation_kind: TranslationKind,
        country: Country,
        gender: Gender,
        how_many: usize,
    ) {
        unimplemented!()
    }

    async fn get_section(&self, index: usize) -> Result<Option<Element>> {
        self.browser.goto("https://www.panda-novel.com/top").await?;

        self.browser
            .wait()
            .for_element(Locator::Css("#panda-app"))
            .await?;

        self.close_annoying_popup().await?;

        let mut sections = self.browser.find_all(Locator::Css(".sections")).await?;
        if index >= sections.len() {
            return Ok(None);
        }

        let section = sections.remove(index);
        Ok(Some(section))
    }

    async fn scroll_up(&self, times: usize) -> Result<()> {
        let actions = KeyActions::new("scroll_up".to_string()).then(KeyAction::Up {
            value: Key::Down.into(),
        });

        for i in 0..times {
            self.browser.perform_actions(actions.clone()).await?;
        }

        Ok(())
    }

    async fn scroll_down(&self, times: usize) -> Result<()> {
        let actions = KeyActions::new("scroll_down".to_string()).then(KeyAction::Down {
            value: Key::Down.into(),
        });

        for i in 0..times {
            self.browser.perform_actions(actions.clone()).await?;
        }

        Ok(())
    }

    async fn close_annoying_popup(&self) -> Result<()> {
        self.browser
            .wait()
            .for_element(Locator::XPath("//div[@class='ui-dialog-icon']"))
            .await?;

        self.browser
            .find(Locator::XPath("//div[@class='ui-dialog-icon']"))
            .await?
            .find(Locator::XPath("//button[@class='btn btn-default']"))
            .await?
            .click()
            .await?;

        Ok(())
    }
}
