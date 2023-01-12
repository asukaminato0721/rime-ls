use librime_sys as librime;
use std::error::Error;
use std::ffi::{CStr, CString};
use std::sync::RwLock;

/// just call unsafe c ffi function simply
/// TODO: make a good rust wrapper
/// FIXME: maybe String not &str, because of LSP backend lifetime
#[derive(Debug)]
pub struct Rime<'t> {
    shared_data_dir: &'t str,
    user_data_dir: &'t str,
    log_dir: &'t str,
}

#[derive(Debug)]
pub struct Candidate {
    pub text: String,
    pub comment: String,
    pub order: usize,
}

impl<'t> Rime<'t> {
    pub fn new(shared_data_dir: &'t str, user_data_dir: &'t str, log_dir: &'t str) -> Rime<'t> {
        Rime {
            shared_data_dir,
            user_data_dir,
            log_dir,
        }
    }

    pub fn init(&self) -> Result<(), Box<dyn Error>> {
        let mut traits: librime::RimeTraits = unsafe { std::mem::zeroed() };
        traits.data_size = std::mem::size_of::<librime::RimeTraits>() as i32; // ?

        // set dirs
        traits.shared_data_dir = CString::new(self.shared_data_dir)?.into_raw();
        traits.user_data_dir = CString::new(self.user_data_dir)?.into_raw();
        traits.log_dir = CString::new(self.log_dir)?.into_raw();
        traits.min_log_level = 1; // WARN

        // set name
        traits.distribution_name = CString::new("Rime")?.into_raw();
        traits.distribution_code_name = CString::new("rime-ls")?.into_raw();
        traits.distribution_version = CString::new(env!("CARGO_PKG_VERSION"))?.into_raw();
        traits.app_name = CString::new("rime.rime-ls")?.into_raw();

        unsafe {
            librime::RimeSetup(&mut traits);
            librime::RimeInitialize(&mut traits);
            if librime::RimeStartMaintenance(false as i32) != 0 {
                librime::RimeJoinMaintenanceThread();
            }
        }

        Ok(())
    }

    pub fn destroy(&self) {
        unsafe {
            librime::RimeFinalize();
        }
    }

    pub fn get_candidates_from_session(
        session_id: usize,
        max_candidates: usize,
    ) -> Result<Vec<Candidate>, Box<dyn Error>> {
        unsafe {
            if librime::RimeFindSession(session_id) == 0 {
                Err("No such session")?
            }
        }
        let mut context: librime::RimeContext = unsafe { std::mem::zeroed() };
        context.data_size = std::mem::size_of::<librime::RimeContext>() as i32; // ?
        unsafe {
            librime::RimeGetContext(session_id, &mut context);
        }
        let res = RwLock::new(Vec::new());
        loop {
            for i in 0..context.menu.num_candidates {
                let candidate = unsafe { *context.menu.candidates.offset(i as isize) };
                let text = unsafe { CStr::from_ptr(candidate.text).to_str()?.to_owned() };
                let comment = unsafe {
                    if candidate.comment.as_ref().is_some() {
                        CStr::from_ptr(candidate.text).to_str()?.to_owned()
                    } else {
                        String::from("")
                    }
                };
                let order = res.read().unwrap().len();
                let cand = Candidate {
                    text,
                    comment,
                    order,
                };
                res.write().unwrap().push(cand);
                if res.read().unwrap().len() >= max_candidates {
                    break;
                }
            }

            if res.read().unwrap().len() >= max_candidates {
                break;
            }
            if context.menu.is_last_page != 0 {
                break;
            }
            // next page
            unsafe {
                if librime::RimeProcessKey(session_id, b'=' as i32, 0) == 0 {
                    break;
                }
                librime::RimeGetContext(session_id, &mut context);
            }
        }
        unsafe {
            librime::RimeFreeContext(&mut context);
        }
        Ok(res.into_inner()?)
    }

    pub fn get_candidates_from_keys(
        &self,
        keys: Vec<u8>,
        max_candidates: usize,
    ) -> Result<Vec<Candidate>, Box<dyn Error>> {
        let session_id = unsafe { librime::RimeCreateSession() };
        unsafe {
            let ck = CString::new(keys)?;
            librime::RimeSimulateKeySequence(session_id, ck.into_raw());
        }
        let res = Rime::get_candidates_from_session(session_id, max_candidates)?;
        unsafe {
            if librime::RimeFindSession(session_id) != 0 {
                librime::RimeDestroySession(session_id);
            }
        }
        Ok(res)
    }

    #[allow(dead_code)]
    /// TODO: simulate typing rather than directly giving string to rime
    pub fn process_key(session_id: usize, key: u8) {
        unsafe {
            if librime::RimeFindSession(session_id) != 0 {
                librime::RimeProcessKey(session_id, key as i32, 0);
            }
        }
    }
}

#[test]
fn test_get_candidates() {
    let shared_data_dir = "/usr/share/rime-data/";
    let user_data_dir = "/home/wlh/.local/share/rime-ls/";
    let log_dir = "/tmp";
    // init
    let rime_ls = Rime::new(shared_data_dir, user_data_dir, log_dir);
    rime_ls.init().unwrap();
    // simulate typing
    let max_candidates = 10;
    let keys = vec![b'w', b'l', b'h'];
    let cands = rime_ls
        .get_candidates_from_keys(keys, max_candidates)
        .unwrap();
    assert_eq!(cands.len(), max_candidates);
    // destroy
    rime_ls.destroy();
}