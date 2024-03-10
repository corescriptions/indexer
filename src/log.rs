const LOG4RS_YAML: &str = "./log4rs.yaml";
const LOG4RS_INIT: &str = r#"refresh_rate: 30 seconds
appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} {l} {m}{n}"
  file:
    kind: file
    path: "logs/insdexer.log"
    encoder:
      pattern: "{d} {l} {m}{n}"
root:
  level: info
  appenders:
    - stdout
    - file
"#;

pub fn init_log() {
    match std::fs::metadata(LOG4RS_YAML) {
        Ok(_) => {}
        Err(_) => {
            std::fs::write(LOG4RS_YAML, LOG4RS_INIT).unwrap();
        }
    }

    log4rs::init_file(LOG4RS_YAML, Default::default()).unwrap();
}
