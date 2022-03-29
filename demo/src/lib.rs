mod pb;
mod utils;

use num_bigint::BigUint;
use substreams::{log, proto, state};

/// Say hello to every first transaction in of a transaction from a block
///
/// `block_ptr`: Pointer of where the block is located in the wasm heap memory
/// `block_len`: Length of the block in wasm heap memory
#[no_mangle]
pub extern "C" fn map_hello_world(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        log::println(format!("Hello, transaction sender: {}", utils::address_pretty(trx.from.as_slice())));
        log::println(format!("Hello, transaction receiver: {}", utils::address_pretty(trx.to.as_slice())));

        substreams::output(trx);
        break;
    }
}

//todo: use erc20-transfer
// have an example of the fetching the number of transfers by block
// maybe an example of the number (counter) of unique contracts in a block (bnb, weth, etc?)

/*
    1. get erc20 transfer => Done
    2. store state ^^ => todo
    3. number of transfer per hour => todo
*/

/// Find and output all the ERC20 transfers
///
/// `block_ptr`: Pointer of where the block is located in the wasm heap memory
/// `block_len`: Length of the block in wasm heap memory
#[no_mangle]
pub extern "C" fn map_erc_20_transfer(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let block: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    let mut transfers = pb::erc20::Transfers { transfers: vec![] };

    for trx in block.transaction_traces {
        for call in trx.calls {
            for log in call.clone().logs {
                if !utils::is_erc20transfer_event(&log) {
                    continue
                }

                // get required values to create transfer event
                let from_addr = &Vec::from(&log.topics[1][12..]);
                let to_addr = &Vec::from(&log.topics[2][12..]);
                let amount = &log.data[0..32];
                let log_ordinal = log.index as u64;

                let transfer_event = pb::erc20::Transfer {
                    from: utils::address_pretty(from_addr.as_slice()),
                    to: utils::address_pretty(to_addr.as_slice()),
                    amount: BigUint::from_bytes_le(amount).to_string(),
                    balance_change_from: utils::find_erc20_storage_changes(&call.clone(), from_addr),
                    balance_change_to: utils::find_erc20_storage_changes(&call.clone(), to_addr),
                    log_ordinal
                };

                transfers.transfers.push(transfer_event);
            }
        }
    }

    substreams::output(transfers);
}

#[no_mangle]
pub extern "C" fn build_erc_20_transfer_state(transfers_ptr: *mut u8, transfers_len: usize) {
    substreams::register_panic_hook();

    let transfers: pb::erc20::Transfers = proto::decode_ptr(transfers_ptr, transfers_len).unwrap();

    for transfer in transfers.transfers {
        state::set(1, format!("transfer:{}:{}", transfer.from, transfer.to),proto::encode(&transfer).unwrap())
    }
}

#[no_mangle]
pub extern "C" fn map_erc_20_transfer_per_hour(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let block: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in block.transaction_traces {

    }
}
