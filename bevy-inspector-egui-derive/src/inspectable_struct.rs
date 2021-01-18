use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::attributes::InspectableAttribute;

pub fn expand_struct(derive_input: &syn::DeriveInput, data: &syn::DataStruct) -> TokenStream {
    let name = &derive_input.ident;
    let id = name;

    let fields = data.fields.iter().enumerate().map(|(i, field)| {
        let ty = &field.ty;

        let field_label = field_label(field, i);
        let accessor = field_accessor(field, i);

        let (builtin_attributes, custom_attributes): (Vec<_>, Vec<_>) = crate::attributes::inspectable_attributes(&field.attrs)
            .partition(InspectableAttribute::is_builtin);
        

        let mut collapse = false;
        for builtin_attribute in builtin_attributes {
            match builtin_attribute {
                InspectableAttribute::Tag(ident) if ident == "collapse" => collapse = true,
                InspectableAttribute::Tag(name) | InspectableAttribute::Assignment(name, _) => panic!("unknown attributes '{}'", name),
            }
        }

        let custom_options = custom_attributes.iter().fold(
            quote! {let mut custom_options = <#ty as bevy_inspector_egui::Inspectable>::FieldOptions::default();},
            |acc,attribute| {
                let assignment = match attribute {
                    InspectableAttribute::Assignment(name, expr) => quote!{ custom_options.#name = #expr; },
                    InspectableAttribute::Tag(name) => quote!{ custom_options.#name = true;}
                };
                quote! {
                    #acc
                    #assignment
                }
            },
        );

        let ui = quote! {
            #custom_options
            let options = bevy_inspector_egui::Options::new(custom_options);
            <#ty as bevy_inspector_egui::Inspectable>::ui(&mut self.#accessor, ui, options);
        };

        let ui = match collapse {
            true => quote! { ui.collapsing(#field_label, |ui| {#ui}); },
            false => ui,
        };

        quote! {
            ui.label(#field_label);
            #ui
            ui.end_row();
        }

    });

    quote! {
        impl bevy_inspector_egui::Inspectable for #name {
            type FieldOptions = ();


            fn ui(&mut self, ui: &mut bevy_inspector_egui::egui::Ui, options: bevy_inspector_egui::Options<Self::FieldOptions>) {
                use bevy_inspector_egui::egui;

                let grid = egui::Grid::new(stringify!(#id));
                grid.show(ui, |ui| {
                    #(#fields)*
                });
            }
        }
    }
}

fn field_accessor(field: &syn::Field, i: usize) -> TokenStream {
    match &field.ident {
        Some(name) => name.to_token_stream(),
        None => syn::Index::from(i).to_token_stream(),
    }
}

fn field_label(field: &syn::Field, i: usize) -> String {
    match &field.ident {
        Some(name) => name.to_string(),
        None => i.to_string(),
    }
}