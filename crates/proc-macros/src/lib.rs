use proc_macro::TokenStream;

// TODO smarter asset system
// We should probably have an `include_asset_static!` and an `include_asset_runtime!` as well as a system for registering assets
#[proc_macro]
pub fn include_asset(input: TokenStream) -> TokenStream {
    let path: syn::LitStr = syn::parse(input).expect("Not a string literal");
    let path = path.value();

    // asset path
    let path = path.strip_prefix("assets/").unwrap_or(&path);

    let assets_path = std::env::var("LUMINOL_ASSETS_PATH").expect("luminol asset path not present");
    let assets_path = std::path::PathBuf::from(assets_path);

    let asset_path = assets_path.join(path);
    let asset_path_str = asset_path.to_string_lossy();

    let tokens = quote::quote! {
        include_bytes!(#asset_path_str)
    };
    tokens.into()
}
