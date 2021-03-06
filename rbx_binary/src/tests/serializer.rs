use rbx_dom_weak::{
    types::{Ref, UDim},
    InstanceBuilder, WeakDom,
};

use crate::{encode, text_deserializer::DecodedModel};

/// A basic test to make sure we can serialize the simplest instance: a Folder.
#[test]
fn just_folder() {
    let tree = WeakDom::new(InstanceBuilder::new("Folder"));
    let mut buffer = Vec::new();

    encode(&tree, &[tree.root_ref()], &mut buffer).expect("failed to encode model");

    let decoded = DecodedModel::from_reader(buffer.as_slice());
    insta::assert_yaml_snapshot!(decoded);
}

/// Ensures that a tree containing some instances with a value and others
/// without will correctly fall back to (some) default value.
#[test]
fn partially_present() {
    let tree = WeakDom::new(InstanceBuilder::new("Folder").with_children(vec![
        // This instance's `Value` property should be preserved.
        InstanceBuilder::new("StringValue").with_property("Value", "Hello"),
        // This instance's `Value` property should be the empty string.
        InstanceBuilder::new("StringValue"),
    ]));

    let root_refs = tree.root().children();

    let mut buffer = Vec::new();
    encode(&tree, root_refs, &mut buffer).expect("failed to encode model");

    let decoded = DecodedModel::from_reader(buffer.as_slice());
    insta::assert_yaml_snapshot!(decoded);
}

/// Ensures that unknown properties get serialized on instances.
#[test]
fn unknown_property() {
    let tree =
        WeakDom::new(InstanceBuilder::new("Folder").with_property("WILL_NEVER_EXIST", "Hi, mom!"));

    let mut buffer = Vec::new();
    encode(&tree, &[tree.root_ref()], &mut buffer).expect("failed to encode model");

    let decoded = DecodedModel::from_reader(buffer.as_slice());
    insta::assert_yaml_snapshot!(decoded);
}

/// Ensures that serializing a tree with an unimplemented property type returns
/// an error instead of panicking.
///
/// This test will need to be updated once we implement the type used here.
#[test]
fn unimplemented_type_known_property() {
    let tree = WeakDom::new(
        InstanceBuilder::new("UIListLayout").with_property("Padding", UDim::new(1.0, -30)),
    );

    let mut buffer = Vec::new();
    let result = encode(&tree, &[tree.root_ref()], &mut buffer);

    assert!(result.is_err());
}

/// Ensures that serializing a tree with an unimplemented property type AND an
/// unknown property descriptor returns an error instead of panicking.
///
/// Because rbx_binary has additional logic for falling back to values with no
/// known property descriptor, we should make sure that logic works.
///
/// This test will need to be updated once we implement the type used here.
#[test]
fn unimplemented_type_unknown_property() {
    let tree = WeakDom::new(
        InstanceBuilder::new("Folder").with_property("WILL_NEVER_EXIST", UDim::new(0.0, 50)),
    );

    let mut buffer = Vec::new();
    let result = encode(&tree, &[tree.root_ref()], &mut buffer);

    assert!(result.is_err());
}

/// Ensures that the serializer returns an error instead of panicking if we give
/// it an ID not present in the tree.
#[test]
fn unknown_id() {
    let tree = WeakDom::new(InstanceBuilder::new("Folder"));

    let mut buffer = Vec::new();
    let result = encode(&tree, &[Ref::new()], &mut buffer);

    assert!(result.is_err());
}
