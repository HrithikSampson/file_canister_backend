use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::caller;
use ic_cdk_macros::{ query, update, init};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};


const SERVICE_ID: u8 = 0 as u8;
#[derive(CandidType, Deserialize, Debug)]
struct FileList{
    file_content_map: HashMap<String,Vec<u8>>,
}
#[derive(CandidType, Deserialize, Debug)]
struct FileInput{
    name: String,
    content: Vec<u8>,
}
#[derive(CandidType, Deserialize, Debug)]
struct FilesWrapper{
    file: HashMap<Principal,FileList>,
}

impl Storable for FilesWrapper {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Bounded { max_size: 10000, is_fixed_size: false };
}

impl FileList{
    fn new() -> Self{
        FileList {
            file_content_map: HashMap::new()
        }
    }
}
#[init]
fn init() {
    let file_wrapper = FilesWrapper {
        file: HashMap::new(),
    };
    FILE_MAPPER.with(|file_ref| {
        file_ref.borrow_mut().insert(SERVICE_ID, file_wrapper);
    });
}
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
 
    static FILE_MAPPER: RefCell<StableBTreeMap<u8, FilesWrapper , VirtualMemory<DefaultMemoryImpl>>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}
#[update]
fn upload_file(file_input: FileInput) -> Result<String, String> {
    let caller = caller();
    FILE_MAPPER.with(|file_ref| {
        
        let mut service = file_ref.borrow_mut().get(&SERVICE_ID).unwrap();
        if service.file.contains_key(&caller) == false {
            service.file.insert(caller, FileList::new());
        }
        if let Some(file) = service.file.get_mut(&caller) { 
            if file.file_content_map.contains_key(&file_input.name) {
                return Err("File name Already Exists".to_string());
            }
            else{
                file.file_content_map.insert(file_input.name,file_input.content);
                
                file_ref.borrow_mut().insert(SERVICE_ID,service);
                return Ok("(done)".to_string());
            }
        }
        else {
            return Err("()".to_string());
        } 
    })
}

#[query]
fn get_file(name: String) -> Vec<u8> {
    let caller = caller();
    FILE_MAPPER.with(|file_ref| {
        let service = file_ref.borrow().get(&SERVICE_ID).unwrap();
        ic_cdk::println!("file:{:#?}",service);
        if let Some(file) = service.file.get(&caller) {
            ic_cdk::println!("file:{:#?}",file);
            if file.file_content_map.contains_key(&name) {
                return file.file_content_map.get(&name).unwrap().to_vec();
            }
            return vec![];
        }
        return vec![];
    })
}

#[query]
fn get_files() -> Vec<FileInput> {
    let caller = caller();
    FILE_MAPPER.with(|file_ref| {
        let service = file_ref.borrow().get(&SERVICE_ID).unwrap();
        if let Some(file) = service.file.get(&caller) {
            return file.file_content_map.clone().into_iter().map(|(key,value)|FileInput{name: key,content: value}).collect();
        }
        return vec![];
    })
}

ic_cdk_macros::export_candid!();