use reqwest::Client;

pub async fn crate_exists(client: &Client, name: &str) -> anyhow::Result<bool> {
    if name.len() > 64 || !name.is_ascii() {
        return Ok(false);
    }

    let name = name.to_ascii_lowercase();

    let url = if name.len() <= 2 {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{}/{}",
            name.len(),
            name
        )
    } else if name.len() == 3 {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/3/{}/{}",
            &name[..1],
            name
        )
    } else {
        format!(
            "https://raw.githubusercontent.com/rust-lang/crates.io-index/master/{}/{}/{}",
            &name[..2],
            &name[2..4],
            name
        )
    };

    let response = client.head(&url).send().await?;
    Ok(response.status().is_success())
}

#[derive(serde::Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub newest_version: String,
    pub crate_size: Option<usize>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub downloads: usize,
    pub recent_downloads: usize,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

pub async fn get_crate_info(client: &Client, name: &str) -> anyhow::Result<CrateInfo> {
    #[derive(serde::Deserialize)]
    struct CratesResponse {
        #[serde(rename = "crate")]
        crate_info: CrateInfo,
    }

    let CratesResponse { crate_info }: CratesResponse = client
        .get(&format!("https://crates.io/api/v1/crates/{}", name))
        .send()
        .await?
        .json()
        .await?;

    Ok(crate_info)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn crate_exists_works() {
        let client = Client::new();
        let crates_and_existence = [
            ("a", true),
            ("at", true),
            ("top", true),
            ("surf", true),
            ("tokio", true),
            ("google-gamesconfiguration1_configuration-cli", true),
            ("_", false),
            ("a_", false),
            ("b!g", false),
            ("g0od", false),
            ("q_e_d", false),
            ("☑-not-an-ascii", false),
            (
                "this_crate_has_so_long_name_that_it_exceeds_64_letters_and_blocked_by_crates_io",
                false,
            ),
        ];
        for &(name, existence) in &crates_and_existence {
            let name_upper = name.to_ascii_uppercase();

            let result = crate_exists(&client, &name).await.ok();
            let result_upper = crate_exists(&client, &name_upper).await.ok();

            assert_eq!(Some(existence), result);
            assert_eq!(Some(existence), result_upper);
        }
    }
}
