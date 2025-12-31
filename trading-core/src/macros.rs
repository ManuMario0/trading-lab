#[macro_export]
macro_rules! define_service {
    // ------------------------------------------------------------------------
    // Entry Point
    // ------------------------------------------------------------------------
    (
        name: $module_name:ident,
        service_type: $service_type_name:literal,
        inputs: { $($inputs:tt)* },
        outputs: { $($outputs:tt)* }
    ) => {
        define_service!(@step_inputs
            // Context (Moved to front for easy access)
            {
                name: $module_name,
                service_type: $service_type_name,
                outputs: { $($outputs)* },
                outputs_id: outputs,
                manager_id: runner_manager,
                state_id: state,
                bindings_id: bindings,
                handler_type: S
            }
            // Accumulators:
            { } // Manifest Entires
            { } // Trait Functions
            { } // Manager Logic
            // Input stream to parse:
            $($inputs)*
        );
    };

    // ------------------------------------------------------------------------
    // Input Parsing: Dispatcher
    // ------------------------------------------------------------------------
    (@step_inputs
        { $($ctx:tt)* } // parsing context
        { $($manifest:tt)* }
        { $($trait_fns:tt)* }
        { $($manager_logic:tt)* }

        // Pattern match: key => fn handler(Type)
        $name:ident => fn $handler:ident($type:ty) $($rest:tt)*
    ) => {
        define_service!(@parse_opts
            { $($ctx)* }
            { $($manifest)* }
            { $($trait_fns)* }
            { $($manager_logic)* }
            { $name }
            { $handler }
            { $type }
            $($rest)*
        );
    };

    // ------------------------------------------------------------------------
    // Option Parsing: Explicit
    // ------------------------------------------------------------------------
    (@parse_opts
        { $($ctx:tt)* }
        { $($manifest:tt)* }
        { $($trait_fns:tt)* }
        { $($manager_logic:tt)* }
        { $name:ident }
        { $handler:ident }
        { $type:ty }

        [ required: $req:literal, variadic: $var:literal ] $(,)? $($rest:tt)*
    ) => {
        define_service!(@accumulate
            { $($ctx)* }
            { $($manifest)* }
            { $($trait_fns)* }
            { $($manager_logic)* }
            name: $name,
            handler: $handler,
            type: $type,
            required: $req,
            variadic: $var,
            rest: { $($rest)* }
        );
    };

    // ------------------------------------------------------------------------
    // Option Parsing: Default
    // ------------------------------------------------------------------------
    (@parse_opts
        { $($ctx:tt)* }
        { $($manifest:tt)* }
        { $($trait_fns:tt)* }
        { $($manager_logic:tt)* }
        { $name:ident }
        { $handler:ident }
        { $type:ty }

        $(,)? $($rest:tt)*
    ) => {
        define_service!(@accumulate
            { $($ctx)* }
            { $($manifest)* }
            { $($trait_fns)* }
            { $($manager_logic)* }
            name: $name,
            handler: $handler,
            type: $type,
            required: true,
            variadic: false,
            rest: { $($rest)* }
        );
    };

    // ------------------------------------------------------------------------
    // Accumulation Step
    // ------------------------------------------------------------------------
    (@accumulate
        {
            name: $module_name:ident,
            service_type: $service_type_name:literal,
            outputs: { $($outputs_def:tt)* },
            outputs_id: $outputs_id:ident,
            manager_id: $manager_id:ident,
            state_id: $state_id:ident,
            bindings_id: $bindings_id:ident,
            handler_type: $handler_type:ident
        }
        { $($manifest:tt)* }
        { $($trait_fns:tt)* }
        { $($manager_logic:tt)* }
        name: $name:ident,
        handler: $handler:ident,
        type: $type:ty,
        required: $req:expr,
        variadic: $var:expr,
        rest: { $($rest:tt)* }
    ) => {
        define_service!(@step_inputs
            {
                name: $module_name,
                service_type: $service_type_name,
                outputs: { $($outputs_def)* },
                outputs_id: $outputs_id,
                manager_id: $manager_id,
                state_id: $state_id,
                bindings_id: $bindings_id,
                handler_type: $handler_type
            }
            {
                $($manifest)*
                $crate::manifest::PortDefinition {
                    name: stringify!($name).to_string(),
                    data_type: stringify!($type).split("::").last().unwrap_or(stringify!($type)).to_string(),
                    required: $req,
                    is_variadic: $var,
                },
            }
            {
                $($trait_fns)*
                fn $handler(&mut self, id: $crate::model::identity::Id, data: $type, outputs: &mut Outputs);
            }
            {
                $($manager_logic)*
                {
                    // Logic to add runner and bindings for this input
                    let outputs_clone = $outputs_id.clone();
                    // Bind identifiers hygienically
                    let name_str = stringify!($name);

                    $manager_id.add_runner::<$handler_type, $type>(
                        name_str,
                        $state_id.clone(),
                        Box::new(move |state: &mut $handler_type, id: $crate::model::identity::Id, data: $type| {
                            if let Ok(mut guard) = outputs_clone.lock() {
                                state.$handler(id, data, &mut *guard);
                            } else {
                                eprintln!("Failed to lock outputs for {}", name_str);
                            }
                        })
                    );

                     if let Some(binding) = $bindings_id.inputs.get(name_str) {
                         $manager_id.update_from_binding(name_str, binding.clone());
                     }
                }
            }
            $($rest)*
        );
    };

    // ------------------------------------------------------------------------
    // Final Step: Generate Module
    // ------------------------------------------------------------------------
    (@step_inputs
        {
            name: $module_name:ident,
            service_type: $service_type_name:literal,
            outputs: { $($out_port_id:ident => $out_type:ty),* $(,)? },
            outputs_id: $outputs_id:ident,
            manager_id: $manager_id:ident,
            state_id: $state_id:ident,
            bindings_id: $bindings_id:ident,
            handler_type: $handler_type:ident
        }
        { $($manifest_entries:expr),* $(,)? } // captured as comma separated expressions
        { $($trait_fns:tt)* }
        { $($manager_logic:tt)* }
        // End of inputs (empty)
    ) => {
        /// Generated Service Definition Module
        pub mod $module_name {
            use super::*;
            use std::sync::{Arc, Mutex};
            use $crate::framework::runner_manager::RunnerManager;
            use $crate::manifest::{ServiceBlueprint, PortDefinition, ServiceBindings, Binding};
            use $crate::model::identity::Id;
            use $crate::comms;

            // 1. Output Handle
            #[allow(dead_code)]
            pub struct Outputs {
                $(
                    pub $out_port_id: $crate::comms::SenderSocket<$out_type>,
                )*
            }

            // 2. Handler Trait
            pub trait Handler: Send + 'static {
                $($trait_fns)*
            }

            // 3. Manifest
            use lazy_static::lazy_static;
            lazy_static! {
                pub static ref MANIFEST: ServiceBlueprint = ServiceBlueprint {
                    service_type: $service_type_name.to_string(),
                    inputs: vec![
                        $($manifest_entries),*
                    ],
                    outputs: vec![
                        $(PortDefinition {
                            name: stringify!($out_port_id).to_string(),
                            data_type: stringify!($out_type).split("::").last().unwrap_or(stringify!($out_type)).to_string(),
                            required: true,
                            is_variadic: false,
                        }),*
                    ],
                };
            }

            // 4. Runner Builder
            pub(crate) fn create_runner_manager<$handler_type>(
                id: Id,
                $bindings_id: ServiceBindings,
                $state_id: Arc<Mutex<$handler_type>>,
            ) -> Result<RunnerManager, String>
            where
                $handler_type: Handler + Send + 'static,
            {
                // Setup Outputs
                let $outputs_id = Arc::new(Mutex::new(Outputs {
                    $(
                        $out_port_id: {
                            let port_name = stringify!($out_port_id);
                            if let Some(Binding::Single(source)) = $bindings_id.outputs.get(port_name) {
                                comms::build_publisher::<$out_type>(&source.address, id)
                                    .map_err(|e| format!("Failed to create publisher for {}: {}", port_name, e))?
                            } else {
                                return Err(format!("Missing binding for output port '{}'", port_name));
                            }
                        },
                    )*
                }));

                // Direct Runner Manager Creation
                let mut $manager_id = RunnerManager::new();

                $($manager_logic)*

                Ok($manager_id)
            }
        }
    };
}
