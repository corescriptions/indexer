use crate::config::{CHECKPOINT_LENGTH, CHECKPOINT_PATH, DB_PATH};
use log::{error, info};
use rocksdb::checkpoint::Checkpoint;
use std::{collections::BTreeSet, fs, path::Path, sync::Mutex};

lazy_static! {
    static ref CHECKPOINT_WORKING: Mutex<bool> = Mutex::new(false);
}

pub fn make_checkpoint_sync(blocknumber: u64) -> Result<(), rocksdb::Error> {
    let secondary_path = Path::new(DB_PATH.as_str()).join("tmp");
    let checkpoint_base_path = Path::new(CHECKPOINT_PATH.as_str());
    let checkpoint_path = checkpoint_base_path.join(&blocknumber.to_string());

    if !checkpoint_base_path.exists() {
        std::fs::create_dir(checkpoint_base_path).unwrap();
    }

    let db = rocksdb::DB::open_as_secondary(
        &rocksdb::Options::default(),
        DB_PATH.as_str(),
        secondary_path.to_str().unwrap(),
    )?;

    let checkpoint = Checkpoint::new(&db).unwrap();
    let result = checkpoint.create_checkpoint(checkpoint_path);

    info!("[checkpoint] new checkpoint: {}", blocknumber);

    result
}

pub fn make_checkpoint(blocknumber: u64) {
    if *CHECKPOINT_WORKING.lock().unwrap() {
        return;
    }

    *CHECKPOINT_WORKING.lock().unwrap() = true;

    delete_earlier_checkpoint();

    tokio::spawn(async move {
        match make_checkpoint_sync(blocknumber) {
            Ok(_) => {}
            Err(e) => {
                error!("[checkpoint] make checkpoint error: {}", e);
            }
        }
        *CHECKPOINT_WORKING.lock().unwrap() = false;
    });
}

pub fn delete_earlier_checkpoint() {
    let checkpoint_base_path = Path::new(CHECKPOINT_PATH.as_str());
    if !checkpoint_base_path.exists() {
        return;
    }

    let mut checkpoints = BTreeSet::new();
    for entry in std::fs::read_dir(checkpoint_base_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if let Ok(checkpoint_blocknumber) = path.file_name().unwrap().to_str().unwrap().parse::<u64>() {
                checkpoints.insert(checkpoint_blocknumber);
            }
        }
    }

    while checkpoints.len() > *CHECKPOINT_LENGTH {
        let first_checkpoint = *checkpoints.first().unwrap();
        let checkpoint_path = checkpoint_base_path.join(first_checkpoint.to_string());
        std::fs::remove_dir_all(checkpoint_path).unwrap();
        checkpoints.remove(&first_checkpoint);
        info!("[checkpoint] delete checkpoint: {}", first_checkpoint);
    }
}

pub fn checkpoints_list() -> Vec<u64> {
    let checkpoint_path = Path::new(CHECKPOINT_PATH.as_str());
    let mut checkpoints = Vec::new();
    let read_dir = std::fs::read_dir(checkpoint_path);
    if read_dir.is_err() {
        return checkpoints;
    }

    for entry in read_dir.unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            if let Ok(checkpoint_blocknumber) = path.file_name().unwrap().to_str().unwrap().parse::<u64>() {
                checkpoints.push(checkpoint_blocknumber);
            }
        }
    }
    
    checkpoints
}

pub fn rollback(blocknumber: u64) -> bool {
    let checkpoint_path = Path::new(CHECKPOINT_PATH.as_str()).join(&blocknumber.to_string());
    if !checkpoint_path.exists() {
        return false;
    }

    {
        let secondary_path = Path::new(DB_PATH.as_str()).join("./.rollback_tmpdb");
        rocksdb::DB::open_as_secondary(&rocksdb::Options::default(), &checkpoint_path, &secondary_path).unwrap();
    }

    fs::remove_dir_all(DB_PATH.as_str()).unwrap();
    fs::create_dir(DB_PATH.as_str()).unwrap();

    for entry in std::fs::read_dir(checkpoint_path).unwrap() {
        let entry = entry.unwrap();
        let entry_type = entry.file_type().unwrap();
        let entry_path = entry.path();
        let dest_path = Path::new(DB_PATH.as_str()).join(entry_path.file_name().unwrap());

        if entry_type.is_file() {
            fs::hard_link(&entry.path(), &dest_path).unwrap();
        }
    }

    true
}
