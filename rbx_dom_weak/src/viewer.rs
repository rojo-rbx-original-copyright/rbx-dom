use std::collections::{BTreeMap, HashMap};

use crate::{
    types::{Ref, Variant},
    WeakDom,
};
use serde::{Deserialize, Serialize};

/// Contains state for viewing and redacting nondeterministic portions of
/// WeakDom objects, making them suitable for usage in snapshot tests.
///
/// `DomViewer` can be held onto and used with a DOM multiple times. IDs will
/// persist when viewing the same instance multiple times, and should stay the
/// same across multiple runs of a test.
pub struct DomViewer {
    referent_map: HashMap<Ref, String>,
    next_referent: usize,
}

impl DomViewer {
    /// Construct a new `DomViewer` with no interned referents.
    pub fn new() -> Self {
        Self {
            referent_map: HashMap::new(),
            next_referent: 0,
        }
    }

    /// View the given `WeakDom`, creating a `ViewedInstance` object that can be
    /// used in a snapshot test.
    pub fn view(&mut self, dom: &WeakDom) -> ViewedInstance {
        let root_referent = dom.root_ref();
        self.populate_referent_map(dom, root_referent);
        self.view_instance(dom, root_referent)
    }

    /// View the children of the root instance of the given `WeakDom`, returning
    /// them as a `Vec<ViewedInstance>`.
    pub fn view_children(&mut self, dom: &WeakDom) -> Vec<ViewedInstance> {
        let root_instance = dom.root();
        let children = root_instance.children();

        for &referent in children {
            self.populate_referent_map(dom, referent);
        }

        children
            .iter()
            .map(|&referent| self.view_instance(dom, referent))
            .collect()
    }

    fn populate_referent_map(&mut self, dom: &WeakDom, referent: Ref) {
        self.referent_map
            .insert(referent, format!("referent-{}", self.next_referent));
        self.next_referent += 1;

        let instance = dom.get_by_ref(referent).unwrap();
        for referent in instance.children() {
            self.populate_referent_map(dom, *referent);
        }
    }

    fn view_instance(&self, dom: &WeakDom, referent: Ref) -> ViewedInstance {
        let instance = dom.get_by_ref(referent).unwrap();

        let children = instance
            .children()
            .iter()
            .copied()
            .map(|referent| self.view_instance(dom, referent))
            .collect();

        let properties = instance
            .properties
            .iter()
            .map(|(key, value)| {
                let key = key.clone();
                let new_value = match value {
                    Variant::Ref(ref_referent) => {
                        let referent_str = self
                            .referent_map
                            .get(ref_referent)
                            .cloned()
                            .unwrap_or_else(|| "[unknown ID]".to_owned());

                        ViewedValue::Ref(referent_str)
                    }
                    other => ViewedValue::Other(other.clone()),
                };

                (key, new_value)
            })
            .collect();

        ViewedInstance {
            referent: self.referent_map.get(&referent).unwrap().clone(),
            name: instance.name.clone(),
            class: instance.class.clone(),
            properties,
            children,
        }
    }
}

/// A transformed view into a `WeakDom` or `Instance` that has been redacted and
/// transformed to be more readable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewedInstance {
    referent: String,
    name: String,
    class: String,
    properties: BTreeMap<String, ViewedValue>,
    children: Vec<ViewedInstance>,
}

/// Wrapper around Variant with refs replaced to be redacted, stable versions of
/// their original IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ViewedValue {
    Ref(String),
    Other(Variant),
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::InstanceBuilder;

    #[test]
    fn redact_single() {
        let dom = WeakDom::new(InstanceBuilder::new("Folder").with_name("Root"));

        insta::assert_yaml_snapshot!(DomViewer::new().view(&dom));
    }

    #[test]
    fn redact_multi() {
        let dom = WeakDom::new(
            InstanceBuilder::new("Folder")
                .with_name("Root")
                .with_children(
                    (0..4)
                        .map(|i| InstanceBuilder::new("Folder").with_name(format!("Child {}", i))),
                ),
        );

        insta::assert_yaml_snapshot!(DomViewer::new().view(&dom));
    }

    #[test]
    fn redact_values() {
        let root = InstanceBuilder::new("ObjectValue").with_name("Root");
        let root_ref = root.referent;
        let root = root.with_property("Value", root_ref);

        let dom = WeakDom::new(root);

        insta::assert_yaml_snapshot!(DomViewer::new().view(&dom));
    }
}
