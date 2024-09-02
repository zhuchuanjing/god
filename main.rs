use tokio::io::AsyncBufReadExt;

use ldk_node::bip39::Mnemonic;
use ldk_node::Builder;
use ldk_node::lightning_invoice::Bolt11Invoice;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::bitcoin::secp256k1::{Secp256k1, PublicKey, SecretKey, hashes::{sha256, Hash}};
use ldk_node::bitcoin::Network;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::RwLock;
use bdk::keys::{DerivableKey, ExtendedKey};
use bdk::{miniscript, Wallet, KeychainKind};
use bdk::database::MemoryDatabase;
use bdk::template::Bip84;
//use bdk::bitcoin::Network;
/*use bdk::database::MemoryDatabase;
use bdk::keys::{DerivableKey, GeneratableKey, GeneratedKey, ExtendedKey, bip39::{Mnemonic, WordCount, Language}};
use bdk::template::Bip84;
use bdk::{miniscript, Wallet, KeychainKind, SyncOptions};
use bdk::blockchain::ElectrumBlockchain;
use bdk::electrum_client::Client;*/

use anyhow::Result;
//#[tokio::main]
//async fn main()-> Result<()> {
mod sled_store;
use sled_store::{SLED, SledRoot, SledStore};
fn main()-> Result<()> {
    let _ = SLED.set(RwLock::new(SledRoot::new("data")));       //set global sled storage
    let mut builder = Builder::new();
    builder.set_log_level(ldk_node::LogLevel::Trace);
    builder.set_log_dir_path(".".into());
    builder.set_storage_dir_path("data".into());
    builder.set_network(Network::Testnet);
    builder.set_esplora_server("https://testnet-electrs.iftas.tech/api".to_string());
    builder.set_gossip_source_rgs("https://rapidsync.lightningdevkit.org/testnet/snapshot".to_string());
    let key: Mnemonic = ldk_node::bip39::Mnemonic::parse_in(bdk::keys::bip39::Language::English,
        "leg practice vendor essay arrest path champion dragon material festival item mobile")?;
    
    builder.set_entropy_bip39_mnemonic(key, None);
    let _ = builder.set_listening_addresses(vec![SocketAddress::from_str("127.0.0.1:8087").unwrap()]);
    let node = //builder.build()?;
    builder.build_with_store(std::sync::Arc::new(SledStore::new("node1")))?;

    node.start()?;
    node.sync_wallets().unwrap();
    println!("{} {:?}", node.node_id(), node.listening_addresses());

    let key_a: Mnemonic = ldk_node::bip39::Mnemonic::parse_in(bdk::keys::bip39::Language::English,
        "smart submit wreck plunge march dial taxi deliver rude ball keen truck")?;
    builder.set_entropy_bip39_mnemonic(key_a, None);
    let _ = builder.set_listening_addresses(vec![SocketAddress::from_str("127.0.0.1:8088").unwrap()]);
    builder.set_log_dir_path("data_a".into());
    builder.build_with_store(std::sync::Arc::new(SledStore::new("node2")))?;
    let node_a = builder.build()?;
    
    node_a.start()?;
    node_a.sync_wallets().unwrap();
    println!("{} {:?}", node_a.node_id(), node_a.listening_addresses());
    //let funding_address = node.onchain_payment().new_address();
    
    println!("{:?}", node.list_balances());
    println!("{:?}", node_a.list_balances());
    println!("{:?}", node.list_channels());
    println!("{:?}", node_a.list_channels());

    let node_a_addr = SocketAddress::from_str("127.0.0.1:8088").unwrap();
    node.connect_open_channel(node_a.node_id(), node_a_addr, 5000, Some(2000), None, false).unwrap();

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    loop {
        let line = runtime.block_on(stdin.next_line());
        match line {
            Ok(Some(line))=> {
                if line == "exit" || line == "quit" {
                    break;
                }
                        //println!("{:?}", console.process(&line));
            }
            _=>{}
        }
    }
    node.stop()?;
    node_a.stop()?;
    Ok(())
}
