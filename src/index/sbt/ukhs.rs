use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use failure::Error;
use lazy_init::Lazy;

use crate::index::sbt::{FromFactory, Node, Update, SBT};
use crate::index::storage::{ReadData, ReadDataError};
use crate::index::{Comparable, Dataset};
use crate::signatures::ukhs::{FlatUKHS, UKHSTrait};
use crate::signatures::{Signature, Signatures};

impl<L> FromFactory<Node<FlatUKHS>> for SBT<Node<FlatUKHS>, L> {
    fn factory(&self, name: &str) -> Result<Node<FlatUKHS>, Error> {
        let data = Lazy::new();
        // TODO: don't hardcode this!
        data.get_or_create(|| FlatUKHS::new(9, 31).unwrap());

        Ok(Node {
            name: name.into(),
            filename: name.into(),
            metadata: HashMap::default(),
            storage: Some(Rc::clone(&self.storage)),
            data: Rc::new(data),
        })
    }
}

impl Update<Node<FlatUKHS>> for Node<FlatUKHS> {
    fn update(&self, _other: &mut Node<FlatUKHS>) -> Result<(), Error> {
        unimplemented!();
    }
}

impl Update<Node<FlatUKHS>> for Dataset<Signature> {
    fn update(&self, other: &mut Node<FlatUKHS>) -> Result<(), Error> {
        // TODO: select the right signatures...
        if let Signatures::UKHS(sig) = &self.data()?.signatures[0] {
            let mut data: FlatUKHS = other.data()?.clone();
            data.merge(sig);

            let new_data = Lazy::new();
            new_data.get_or_create(|| data);

            mem::replace(&mut other.data, Rc::new(new_data));
            return Ok(());
        }
        unimplemented!()
    }
}

impl Comparable<Node<FlatUKHS>> for Node<FlatUKHS> {
    fn similarity(&self, other: &Node<FlatUKHS>) -> f64 {
        let o_sig: &FlatUKHS = other.data().unwrap();
        let me_sig: &FlatUKHS = self.data().unwrap();
        1.0 - me_sig.distance(o_sig)
    }

    fn containment(&self, _other: &Node<FlatUKHS>) -> f64 {
        unimplemented!();
    }
}

impl Comparable<Dataset<Signature>> for Node<FlatUKHS> {
    fn similarity(&self, other: &Dataset<Signature>) -> f64 {
        let odata = other.data().unwrap();

        if odata.signatures.len() > 1 {
            // TODO: select the right signatures...
            unimplemented!()
        } else if let Signatures::UKHS(o_sig) = &odata.signatures[0] {
            let me_sig: &FlatUKHS = self.data().unwrap();
            1.0 - me_sig.distance(o_sig)
        } else {
            // TODO: sig[0] was not a UKHS
            unimplemented!()
        }
    }

    fn containment(&self, _other: &Dataset<Signature>) -> f64 {
        unimplemented!();
    }
}

impl ReadData<FlatUKHS> for Node<FlatUKHS> {
    fn data(&self) -> Result<&FlatUKHS, Error> {
        if let Some(storage) = &self.storage {
            Ok(self.data.get_or_create(|| {
                let raw = storage.load(&self.filename).unwrap();
                FlatUKHS::from_reader(&mut &raw[..]).unwrap()
            }))
        } else if let Some(data) = self.data.get() {
            Ok(data)
        } else {
            Err(ReadDataError::LoadError.into())
        }
    }
}
