use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_http_server::ServerBuilder;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use lammes_automata_theory::Dfa;
use std::collections::{HashMap, HashSet};
use std::cmp::min;

/// Holds all methods which are callable over this RCP server.
#[rpc]
pub trait Rpc {
    /// Delegates to the check method in the lammes_automata_theory library crate.
    /// The documentation can be found there.
    #[rpc(name = "check")]
    fn check(&self, dfa: Dfa, input: String) -> Result<(bool, Vec<String>)>;

    /// Calls the minimize method of the lammes_automata_theory library crate and improves the output.
    /// The minimize method returns a map with all renaming operations, mapping the old names to the new names.
    /// But for our client it might be more useful to have a list of all old names for each merged new name.
    /// Example: q0 and q1 are equivalent and merged into q0. This method will have a map that maps the
    /// new name q0 to all old names, namely q0 and q1.
    #[rpc(name = "minimize")]
    fn minimize(&self, dfa: Dfa) -> Result<(Dfa, HashMap<String, HashSet<String>>)>;
}

pub struct RpcImpl;
impl Rpc for RpcImpl {
    fn check(&self, dfa: Dfa, input: String) -> Result<(bool, Vec<String>)> {
        Ok(dfa.check(input.as_str()))
    }

    fn minimize(&self, dfa: Dfa) -> Result<(Dfa, HashMap<String, HashSet<String>>)> {
        let mut minimized_dfa = dfa.clone();
        let renaming_operations = minimized_dfa.minimize();
        // The renaming_operations maps every old name to the new name.
        // But we want every new name mapped to every old name that belongs to that new name.
        // Example: The state name "q0" has been merged from the old state names "q0", "q1" and "q2".
        let mut old_names_by_their_new_names: HashMap<String, HashSet<String>> = HashMap::new();
        // Insert every renaming operation into our new map.
        for (old_name, new_name) in renaming_operations {
            match old_names_by_their_new_names.get_mut(new_name.as_str()) {
                Some(old_names_by_new_name) => {
                    old_names_by_new_name.insert(old_name);
                },
                // Create a new set for the new name if none yet exists.
                None => {
                    old_names_by_their_new_names.insert(new_name.clone(), HashSet::new());
                    old_names_by_their_new_names.get_mut(new_name.as_str()).unwrap().insert(old_name);
                }
            }
        }
        Ok((minimized_dfa, old_names_by_their_new_names))
    }
}

/// Starts a server that exposes the functionality of the [lammes_automata_theory library crate](https://github.com/simon-lammes/lammes_automata_theory)
/// via HTTP, using the JSON-RCP specifications. The server library can be
/// found [here.](https://github.com/paritytech/jsonrpc)
fn main() {
    let mut io = IoHandler::new();
    // Register the procedures that should be callable via RPC.
    io.extend_with(RpcImpl.to_delegate());
    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:3030".parse().unwrap())
        .unwrap();
    server.wait();
}
