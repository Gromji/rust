#![cfg_attr(doc, allow(internal_features))]
#![cfg_attr(doc, feature(rustdoc_internals))]
#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_symbol_mangling;
extern crate rustc_target;
extern crate rustc_ty_utils;
extern crate stable_mir;

use rustc_codegen_ssa::traits::CodegenBackend;
use rustc_codegen_ssa::{CodegenResults, CrateInfo};
use rustc_data_structures::fx::FxIndexMap;
use rustc_session::Session;
use std::any::Any;

pub struct CCodegenBackend(());

impl CodegenBackend for CCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        ""
    }

    fn init(&self, _sess: &Session) {}

    #[allow(unused)]
    fn codegen_crate<'tcx>(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        metadata: rustc_metadata::EncodedMetadata,
        need_metadata_module: bool,
    ) -> Box<dyn Any> {
        Box::new(CodegenResults {
            modules: vec![],
            allocator_module: None,
            metadata_module: None,
            metadata,
            crate_info: CrateInfo::new(tcx, "none".to_string()),
        })
    }

    #[allow(unused)]
    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &rustc_session::Session,
        outputs: &rustc_session::config::OutputFilenames,
    ) -> (
        rustc_codegen_ssa::CodegenResults,
        rustc_data_structures::fx::FxIndexMap<
            rustc_middle::dep_graph::WorkProductId,
            rustc_middle::dep_graph::WorkProduct,
        >,
    ) {
        let codegen_results = ongoing_codegen
            .downcast::<CodegenResults>()
            .expect("in join_codegen: ongoing_codegen is not a CodegenResults");
        (*codegen_results, FxIndexMap::default())
    }

    #[allow(unused)]
    fn link(
        &self,
        sess: &rustc_session::Session,
        codegen_results: rustc_codegen_ssa::CodegenResults,
        outputs: &rustc_session::config::OutputFilenames,
    ) -> Result<(), rustc_span::ErrorGuaranteed> {
        use rustc_session::{
            config::{CrateType, OutFileName},
            output::out_filename,
        };
        use std::io::Write;
        let crate_name = codegen_results.crate_info.local_crate_name;
        let output_name = out_filename(sess, CrateType::Executable, &outputs, crate_name);
        match output_name {
            OutFileName::Real(ref path) => {
                let mut out_file = ::std::fs::File::create(path).unwrap();
                write!(out_file, "This has been \"compiled\" successfully.").unwrap();
            }
            OutFileName::Stdout => {
                let mut stdout = std::io::stdout();
                write!(stdout, "This has been \"compiled\" successfully.").unwrap();
            }
        }
        Ok(())
    }
}

// Calling the compiler with -Zcodegen-backend=<path to this codegen> will hot plug this backend
#[no_mangle]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(CCodegenBackend(()))
}
