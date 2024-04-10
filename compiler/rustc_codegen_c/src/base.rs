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

use rustc_codegen_ssa::{CodegenResults, CompiledModule, CrateInfo};
use rustc_metadata::EncodedMetadata;
use rustc_middle::mir::mono::{CodegenUnit, MonoItem};
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_session::config::{OutputFilenames, OutputType};
use std::io::Write;

pub struct Context {
    code: String,
}

impl Context {
    pub fn new() -> Self {
        Self { code: String::new() }
    }

    pub fn push(&mut self, code: &str) {
        self.code.push_str(code);
    }

    pub fn push_line(&mut self, code: &str) {
        self.code.push_str(code);
        self.code.push_str("\n");
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }
}

pub struct OngoingCodegen {
    context: Context,
}

impl OngoingCodegen {
    pub fn join(
        &self,
        name: String,
        ongoing_codegen: &OngoingCodegen,
        metadata: EncodedMetadata,
        crate_info: CrateInfo,
        output_files: &OutputFilenames,
    ) -> CodegenResults {
        let path = output_files.temp_path(OutputType::Object, Some(name.as_str()));

        let code = ongoing_codegen.context.get_code();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(code.as_bytes()).unwrap();

        let modules = vec![CompiledModule {
            name: name,
            kind: rustc_codegen_ssa::ModuleKind::Regular,
            object: Some(path),
            bytecode: None,
            dwarf_object: None,
        }];

        CodegenResults {
            crate_info: crate_info,
            modules: modules,
            allocator_module: None,
            metadata_module: None,
            metadata: metadata,
        }
    }
}

fn transpile_cgu<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    cgu: &CodegenUnit<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
) {
    for (item, _data) in cgu.items() {
        if item.def_id().krate != 0u32.into() {
            continue;
        }

        match item {
            MonoItem::Fn(inst) => {
                let mir = tcx.instance_mir(inst.def);
                with_no_trimmed_paths!({
                    let mut buf = Vec::new();

                    rustc_middle::mir::pretty::write_mir_fn(tcx, mir, &mut |_, _| Ok(()), &mut buf)
                        .unwrap();
                    // write!(std::io::stdout(), "{}", String::from_utf8_lossy(&buf).into_owned())
                    //     .unwrap()
                    ongoing_codegen.context.push_line(&String::from_utf8_lossy(&buf).into_owned());
                });
            }
            MonoItem::Static(def) => {
                // write!(std::io::stdout(), "DEF: {:?}", def).unwrap()
                ongoing_codegen.context.push_line(&format!("static {};", tcx.def_path_str(def),));
            }
            MonoItem::GlobalAsm(item_id) => {
                // write!(std::io::stdout(), "ITEM: {:?}", tcx.hir().item(*item_id)).unwrap();
                ongoing_codegen
                    .context
                    .push_line(&format!("asm!(\"{:?}\";", tcx.hir().item(*item_id),));
            }
        }
    }
}

pub fn run<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    metadata: rustc_metadata::EncodedMetadata,
) -> Box<(String, OngoingCodegen, EncodedMetadata, CrateInfo)> {
    let cgus: Vec<_> = tcx.collect_and_partition_mono_items(()).1.iter().collect();
    let mut ongoing_codegen = OngoingCodegen { context: Context::new() };

    for cgu in &cgus {
        transpile_cgu(tcx, cgu, &mut ongoing_codegen);
    }

    let name: String = cgus.iter().next().unwrap().name().to_string();

    Box::new((name, ongoing_codegen, metadata, CrateInfo::new(tcx, "c".to_string())))
}
