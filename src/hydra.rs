use anyhow::{Result, anyhow, bail};
use reqwest::{Client, ClientBuilder, header};
use scraper::{Html, Selector};
use serde::Deserialize;

pub struct HydraApi {
    client: Client,
    base_url: &'static str,
}

impl HydraApi {
    /// Create a new instance of this struct.
    /// All requests sent to Hydra will have a custom User-Agent header to properly identify this
    /// project.
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                .user_agent("github:diogotcorreia/nixpkgs-build-failure-notifier")
                .build()
                .expect("reqwest client to be created successfully"),
            base_url: "https://hydra.nixos.org",
        }
    }

    pub async fn get_latest_build(&self, jobset: &str, job: &str) -> Result<HydraBuild> {
        let res = self
            .client
            .get(format!("{}/job/{}/{}", self.base_url, jobset, job))
            .send()
            .await?;

        let text = res.text().await?;
        let dom = Html::parse_document(&text);
        let selector_warning = Selector::parse(".alert.alert-warning").unwrap();

        if dom.select(&selector_warning).next().is_some() {
            bail!("Job {}/{} is not part of latest evaluation", jobset, job);
        }

        let selector = Selector::parse("#tabs-status table td a.row-link").unwrap();

        let build_id: u64 = dom
            .select(&selector)
            .next()
            .map(|element_ref| {
                element_ref.text().fold(String::new(), |mut a, b| {
                    a.reserve(b.len());
                    a.push_str(b);
                    a
                })
            })
            .and_then(|element| element.parse().ok())
            .ok_or(anyhow!(
                "Failed to find latest build for job {}/{}",
                jobset,
                job
            ))?;

        Ok(self
            .client
            .get(format!("{}/build/{}", self.base_url, build_id))
            .header(header::ACCEPT, "application/json")
            .send()
            .await?
            .json()
            .await?)
    }
}

impl Default for HydraApi {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
pub struct HydraBuild {
    pub id: u64,
    pub buildstatus: u8,
    pub project: String,
    pub jobset: String,
    pub job: String,
    pub nixname: String,
}

impl HydraBuild {
    pub fn build_status_to_str(&self) -> &'static str {
        // https://github.com/NixOS/hydra/blob/34ff66a460c21ee69d840c8c896d067405ba4a3e/src/root/common.tt#L235-L257
        match self.buildstatus {
            0 => "Succeeded",
            1 => "Failed",
            2 => "Dependency Failed",
            3 | 9 => "Aborted",
            4 => "Cancelled",
            6 => "Failed with output",
            7 => "Timed out",
            10 => "Log limit exceeded",
            11 => "Output size limit exceeded",
            12 => "Non-deterministic build",
            _ => "Failed (unknown)",
        }
    }

    pub fn is_failing(&self) -> bool {
        self.buildstatus != 0
    }

    pub fn get_full_name(&self) -> String {
        format!("{}:{}:{}", self.project, self.jobset, self.job)
    }
}
