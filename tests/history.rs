#![type_length_limit = "5000000"]

extern crate krill;

/// This tests regressions for the Command history and details as exposed through the
/// Krill API.
///
/// This uses commands and history as generated by other functional tests defined
/// here.
#[tokio::test]
#[cfg(feature = "functional-tests")]
async fn history() {
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::{env, fs};

    use krill::commons::api::{CaCommandDetails, CommandHistoryCriteria, Handle};
    use krill::commons::crypto::KrillSigner;
    use krill::commons::util::file;
    use krill::daemon::ca::CaServer;
    use krill::daemon::mq::EventQueueListener;
    use krill::test::*;

    const KRILL_HISTORY_JSON_GENERATE: &str = "KRILL_HISTORY_JSON_GENERATE";

    // Run this test with ENV variable KRILL_HISTORY_JSON_GENERATE = 1 in order to generate json
    // for missing files
    assert_scenario("ca_embedded", &["ta", "child"]).await;
    assert_scenario("ca_roas", &["ta", "child"]).await;
    assert_scenario("ca_rfc6492", &["ta", "rfc6492"]).await;
    assert_scenario("ca_keyroll_rfc6492", &["ta", "rfc6492"]).await;
    assert_scenario("ca_grandchildren", &["ta", "CA1", "CA2", "CA3", "CA4"]).await;
    assert_scenario("remote_publication", &["ta", "child"]).await;

    async fn assert_scenario(scenario: &str, cas: &[&str]) {
        let d = tmp_dir();
        let server = make_server(&d, scenario).await;

        for ca in cas {
            let handle = Handle::from_str(ca).unwrap();
            assert_history(&server, scenario, &handle).await;
        }

        let _ = fs::remove_dir_all(d);
    }

    async fn make_server(work_dir: &PathBuf, scenario: &str) -> CaServer {
        let mut source = PathBuf::from("test-resources/api/regressions/v0_6_0/history/");
        source.push(scenario);
        source.push("cas");

        let mut server_dir = work_dir.clone();
        server_dir.push(scenario);

        let mut server_cas_dir = server_dir.clone();
        server_cas_dir.push("cas");
        file::backup_dir(&source, &server_cas_dir).unwrap();

        let signer = KrillSigner::build(&server_dir).unwrap();

        let event_queue = Arc::new(EventQueueListener::default());

        CaServer::build(&server_dir, None, None, event_queue, signer)
            .await
            .unwrap()
    }

    async fn assert_history(server: &CaServer, scenario: &str, ca: &Handle) {
        let crit_dflt = CommandHistoryCriteria::default();
        let history = server.get_ca_history(ca, crit_dflt).await.unwrap();

        let mut expexted_file = PathBuf::from("test-resources/api/regressions/v0_6_0/history/");
        expexted_file.push(scenario);
        expexted_file.push("expected");
        expexted_file.push(ca.as_str());
        expexted_file.push("history.json");

        if let Ok(bytes) = file::read(&expexted_file) {
            let expected_history = serde_json::from_slice(bytes.as_ref()).unwrap();
            assert_eq!(history, expected_history);
        } else {
            let content = serde_json::to_string_pretty(&history).unwrap();

            if env::var(KRILL_HISTORY_JSON_GENERATE).is_ok() {
                file::save(content.as_bytes(), &expexted_file).unwrap();
            } else {
                panic!(
                    "Expected file: {} with json:\n{}",
                    expexted_file.to_string_lossy().to_string(),
                    content
                );
            }
        }

        for record in history.commands() {
            let key = record.command_key().unwrap();

            let mut expected_command_file = PathBuf::from("test-resources/api/regressions/v0_6_0/history/");
            expected_command_file.push(scenario);
            expected_command_file.push("expected");
            expected_command_file.push(ca.as_str());
            expected_command_file.push(&format!("{}.json", key));

            let details = server.get_ca_command_details(&ca, key).unwrap();

            if let Ok(bytes) = file::read(&expected_command_file) {
                let expected_details: CaCommandDetails = serde_json::from_slice(bytes.as_ref()).unwrap();
                assert_eq!(details, expected_details);
            } else {
                let content = serde_json::to_string_pretty(&details).unwrap();
                if env::var(KRILL_HISTORY_JSON_GENERATE).is_ok() {
                    file::save(content.as_bytes(), &expected_command_file).unwrap();
                } else {
                    panic!(
                        "Expected file: {} with json:\n{}",
                        expected_command_file.to_string_lossy().to_string(),
                        content
                    );
                }
            }
        }
    }
}
