use crate::config::OPEN_FILES_LIMIT;
use log::info;

pub fn adjust_open_files_limit() {
    let limit = *OPEN_FILES_LIMIT;
    if limit == 0 {
        return;
    }

    let mut rlimit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit) };
    info!("current open files limit: {}", rlimit.rlim_cur);

    let new_limit = libc::rlimit {
        rlim_cur: limit,
        rlim_max: limit,
    };
    unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &new_limit) };

    unsafe {
        libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit);
    }
    info!("new open files limit: {}", rlimit.rlim_cur);
}
