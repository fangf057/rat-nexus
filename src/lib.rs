//! RatatUI scaffolding library with procedural macros.
//!
//! This library provides the `#[component]` and `#[router]` macros to simplify
//! component and router implementations.

#![allow(dead_code)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, ItemEnum};

/// Derive macro for implementing the `Component` trait.
///
/// # Usage
///
/// ```ignore
/// #[component]
/// struct MyComponent {
///     state: SharedState<AppState>,
/// }
/// ```
///
/// The macro generates:
/// - `impl Component for MyComponent`
/// - A default `render` method that renders a simple placeholder.
/// - A default `handle_event` method that returns `None`.
/// You can override these by providing your own implementations.
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let expanded = quote! {
        #input

        impl ::rat_setup::component::Component for #struct_name {
            fn render(&mut self, frame: &mut ::ratatui::prelude::Frame, cx: &mut ::rat_setup::application::Context<Self>) {
                // Default implementation: render a placeholder
                let text = format!("Component {} (implement render)", stringify!(#struct_name));
                let paragraph = ::ratatui::widgets::Paragraph::new(text)
                    .alignment(::ratatui::layout::Alignment::Center);
                frame.render_widget(paragraph, cx.area);
            }

            fn handle_event(&mut self, event: ::rat_setup::component::traits::Event, cx: &mut ::rat_setup::application::EventContext<Self>) -> Option<::rat_setup::component::traits::Action> {
                // Default implementation: no handling
                let _ = event;
                let _ = cx;
                None
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for implementing the `Router` trait.
///
/// # Usage
///
/// ```ignore
/// #[router]
/// enum AppRouter {
///     PageA(PageA),
///     PageB(PageB),
/// }
/// ```
///
/// The macro generates:
/// - `impl Router for AppRouter`
/// - Methods `navigate`, `current_route`, `current_component` based on enum variants.
#[proc_macro_attribute]
pub fn router(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_name = &input.ident;
    let variants = &input.variants;

    // Generate match arms for current_component
    let match_arms: Vec<_> = variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            #enum_name::#variant_name(comp) => comp,
        }
    }).collect();

    // Generate route strings for navigate
    let route_arms: Vec<_> = variants.iter().map(|v| {
        let variant_name = &v.ident;
        let route = variant_name.to_string().to_lowercase();
        quote! {
            #route => #enum_name::#variant_name(Default::default()),
        }
    }).collect();

    let expanded = quote! {
        #input

        impl ::rat_setup::router::Router for #enum_name {
            fn navigate(&mut self, route: ::rat_setup::router::traits::Route) {
                // For simplicity, we just replace self with a new instance.
                // In a real implementation you might want to keep the same instance.
                *self = match route.as_str() {
                    #(#route_arms)*
                    _ => return,
                };
            }

            fn current_route(&self) -> &::rat_setup::router::traits::Route {
                // This is a dummy implementation; you'd need to store the route.
                // For simplicity, we return a static string.
                static CURRENT: String = String::new();
                &CURRENT
            }

            fn current_component(&mut self) -> &mut dyn ::rat_setup::component::Component {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}