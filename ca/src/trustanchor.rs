use resources::ResourceSet;
use krill_commons::api::admin::CaHandle;
use krill_commons::eventsourcing::{StoredEvent, SentCommand, CommandDetails, Aggregate};

#[allow(dead_code)]
const TA_NS: &str = "trustanchors";
const TA_ID: &str = "ta";

pub fn ta_handle() -> CaHandle {
    CaHandle::from(TA_ID)
}

//------------ TrustAnchorInit -----------------------------------------------

pub type TrustAnchorInit = StoredEvent<TrustAnchorInitDetails>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrustAnchorInitDetails {
    resources: ResourceSet
}

impl TrustAnchorInitDetails {
    pub fn with_all_resources(handle: &CaHandle) -> TrustAnchorInit {
        let details = {
            let asns = "AS0-AS4294967295";
            let v4 = "0.0.0.0/0";
            let v6 = "::/0";
            let resources = ResourceSet::from_strs(asns, v4, v6).unwrap();
            TrustAnchorInitDetails { resources }
        };

        TrustAnchorInit::new(handle.as_ref(), 0, details)
    }
}


//------------ TrustAnchorEvent ----------------------------------------------

pub type TrustAnchorEvent = StoredEvent<TrustAnchorEventDetails>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrustAnchorEventDetails;


//------------ TrustAnchorCommand --------------------------------------------

pub type TrustAnchorCommand = SentCommand<TrustAnchorCommandDetails>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrustAnchorCommandDetails;

impl CommandDetails for TrustAnchorCommandDetails {
    type Event = TrustAnchorEvent;
}


//------------ TrustAnchor ---------------------------------------------------

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TrustAnchor {
    id: CaHandle,
    version: u64,
    resources: ResourceSet
}

impl Aggregate for TrustAnchor {
    type Command = TrustAnchorCommand;
    type Event = TrustAnchorEvent;
    type InitEvent = TrustAnchorInit;
    type Error = Error;

    fn init(event: Self::InitEvent) -> Result<Self, Self::Error> {
        let (id, _version, init) = event.unwrap();
        let id = CaHandle::from(id);
        let version = 1; // after applying init
        let resources = init.resources;

        Ok(TrustAnchor { id, version, resources })
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply(&mut self, _event: Self::Event) {
        unimplemented!()
    }

    fn process_command(&self, _command: Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        unimplemented!()
    }
}



#[derive(Clone, Debug, Display)]
#[display(fmt = "Trust Anchor Issue")]
pub struct Error;

impl std::error::Error for Error {}

//------------ Tests ---------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;
    use krill_commons::util::test;
    use krill_commons::eventsourcing::AggregateStore;
    use krill_commons::eventsourcing::DiskAggregateStore;

    #[test]
    fn should_init_ta() {
        test::test_with_tmp_dir(|d| {
            let store = DiskAggregateStore::<TrustAnchor>::new(&d, TA_NS).unwrap();
            let handle = ta_handle();
            let init = TrustAnchorInitDetails::with_all_resources(&handle);

            store.add(handle.as_ref(), init).unwrap();
        });
    }

}