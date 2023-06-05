use ureq::Agent;

/// The client for accessing TFT data.
pub struct Client {
    /// The stored API key for your app.
    api_key: String,
    /// The agent used to make HTTP requests.
    agent: Agent,
}

impl Client {
    pub fn new(api_key: String) -> Client {
        Client {
            api_key,
            agent: Agent::new(),
        }
    }

    pub fn new_with_agent(api_key: String, agent: Agent) -> Client {
        Client { api_key, agent }
    }
}
