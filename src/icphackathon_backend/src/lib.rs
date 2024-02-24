use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::call;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::BTreeMap;
use std::{cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 1024; 
#[derive(CandidType, Deserialize, Storable, Debug)]
struct IotiDevice {
    id: String,
    name: String,
    owner: Principal,
    data: Vec<u8>,
}

#[derive(CandidType, Deserialize, Debug)]
enum ElectricityEventError {
    IllegalElectricityEvent,
    DataNotReceived,
}

#[derive(CandidType, Deserialize, Debug)]
struct ElectricityEvent {
    id: String,
    timestamp: u64,
    voltage: u64,
    current: u64,
    value: u64,
}

impl Storable for ElectricityEvent {
    fn to_bytes(&self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: Vec<u8>) -> ElectricityEvent {
        Decode!(&bytes, ElectricityEvent).unwrap()
    }
}

impl BoundedStorable for ElectricityEvent {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static ELECTRICITY_EVENT_MAP: RefCell<StableBTreeMap<u64, ElectricityEvent, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        ));
}

#[ic_cdk::query]
fn get_electricity_events() -> Result<Vec<ElectricityEvent>, String> {
    ELECTRICITY_EVENT_MAP
        .try_with(|map|{Ok (map.borrow().iter().map(|(_, v)| v.clone()).collect::<Vec<_>>())
})
 } 

#[derive(CandidType, Deserialize, Debug)]
enum LeakageStatus {
    NoLeakage,
    LeakageDetected,
}

#[ic_cdk::query]
fn detect_electrical_leakage() -> LeakageStatus {
    let events = ELECTRICITY_EVENT_MAP.with(|map| map.borrow().values().cloned().collect::<Vec<_>>());

    for event in events {
        
        if event.current > 100 || event.voltage > 200 {
            return LeakageStatus::LeakageDetected;
        }
    }

    LeakageStatus::NoLeakage
}

#[ic_cdk::update]
fn update_electricity_events() -> Result<(), String> {
    let api_url = "https://mockapi.io/clone/65d8af7ac96fbb24c1bc1bf8/electricityevent";
    
    let response = reqwest::blocking::get(api_url).map_err(|e| format!("Failed to fetch data: {}", e))?;
    let events: Vec<ElectricityEvent> = response.json().map_err(|e| format!("Failed to parse JSON: {}", e))?;

    ELECTRICITY_EVENT_MAP.with(|map| {
        let mut map = map.borrow_mut();
        map.clear();
        for event in events {
            map.insert(event.timestamp, event);
        }
    });

    Ok(())
}

