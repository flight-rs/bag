#[macro_use]
extern crate quote;
extern crate grabber;

use grabber::{Registry, Crate, Preflight, Solver, Artifact, packmode};

pub fn register(reg: &mut Registry) {
    reg.add_target(|solve: &mut Solver| {
        solve.check_artifact_mime("image/*");
        let addr = solve.artifact_reference(packmode::Web);
        solve.output(quote! {{
            let img = #cra::web::html_element::ImageElement::new();
            img.set_src(#addr);
            img
        }});
    })
    .require_type("ImageElement")
    .provides("stdweb")
    .provides("html-image")
    .description(include_str!("image-element.md"))
    .require_crate(Crate::name_range("stdweb", "0.3.0", "0.5.0"));

    let mut pre = Preflight::new();
    pre;
    reg.add_packer("cargo-web", packmode::Web, |pack: &mut Packer| {
        use std::path::Path;

        let root = Name::new(pack.arg("public_url").unwrap_or("/"))?;
        let path = Name::file(pack.put_file(dep.pack_root().join("static"))?);
        pack.reference(path.within(root)?)?;
    })
    .provides("cargo-web")
    .description(include_str!("cargo-web.md"))
    .require_env_err("COMPILING_UNDER_CARGO_WEB", "1", err_msg("must run with cargo-web"));
}
