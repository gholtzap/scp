use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NfProfile {
    pub nf_instance_id: String,
    pub nf_type: String,
    pub nf_status: String,
    pub ipv4_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NfDiscoveryParams {
    pub target_nf_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requester_nf_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_names: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub nf_instances: Vec<NfProfile>,
}

pub struct NrfClient {
    client: Client,
    nrf_uri: String,
}

impl NrfClient {
    pub fn new(nrf_uri: String, client: Client) -> Self {
        Self { client, nrf_uri }
    }

    pub async fn register(&self, profile: &NfProfile) -> Result<NfProfile> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.nrf_uri, profile.nf_instance_id
        );

        let response = self
            .client
            .put(&url)
            .json(profile)
            .send()
            .await
            .context("Failed to send registration request to NRF")?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let registered_profile: NfProfile = response
                    .json()
                    .await
                    .context("Failed to parse NRF registration response")?;

                tracing::info!(
                    "Successfully registered NF instance {} with NRF",
                    profile.nf_instance_id
                );

                Ok(registered_profile)
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF registration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn discover(&self, params: &NfDiscoveryParams) -> Result<Vec<NfProfile>> {
        let url = format!("{}/nnrf-disc/v1/nf-instances", self.nrf_uri);

        let response = self
            .client
            .get(&url)
            .query(&[("target-nf-type", &params.target_nf_type)])
            .send()
            .await
            .context("Failed to send discovery request to NRF")?;

        match response.status() {
            StatusCode::OK => {
                let result: SearchResult = response
                    .json()
                    .await
                    .context("Failed to parse NRF discovery response")?;

                Ok(result.nf_instances)
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF discovery failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn deregister(&self, nf_instance_id: &str) -> Result<()> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.nrf_uri, nf_instance_id
        );

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send deregistration request to NRF")?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => {
                tracing::info!("Successfully deregistered NF instance {} from NRF", nf_instance_id);
                Ok(())
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF deregistration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }
}
