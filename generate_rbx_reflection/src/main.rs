mod roblox_install;
mod api_dump;

use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    error::Error,
};

use quote::quote;
use proc_macro2::Literal;
use lazy_static::lazy_static;

use crate::api_dump::{Dump, DumpClassMember};

lazy_static! {
    static ref OUTPUT_DIR: PathBuf = {
        let mut output = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output.pop();
        output.push("rbx_reflection");
        output.push("src");
        output
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Output at {}", OUTPUT_DIR.display());

    let dump = Dump::read()?;

    let classes = dump.classes.iter().map(|class| {
        let class_name = Literal::string(&class.name);

        let properties = class.members.iter().filter_map(|member|
            match member {
                DumpClassMember::Property { name, value_type } => {
                    let member_name = Literal::string(&name);
                    let value_type_name = Literal::string(&value_type.name);

                    Some(quote! {
                        properties.insert(#member_name, RbxInstanceProperty {
                            name: #member_name,
                            value_type: #value_type_name,
                        });
                    })
                },
                _ => None,
            }
        );

        quote! {
            output.insert(#class_name, RbxInstanceClass {
                name: #class_name,
                properties: {
                    #[allow(unused_mut)]
                    let mut properties = HashMap::new();
                    #(#properties)*
                    properties
                },
            });
        }
    });

    let output = quote! {
        #![allow(unused_mut)]
        use std::collections::HashMap;
        use crate::types::*;

        pub fn get_instances() -> HashMap<&'static str, RbxInstanceClass> {
            let mut output = HashMap::new();

            #(#classes)*

            output
        }
    };

    let mut file = File::create(OUTPUT_DIR.join("dump.rs"))?;
    write!(file, "{}", output)?;

    Ok(())
}