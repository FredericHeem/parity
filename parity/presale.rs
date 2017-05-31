// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use ethcore::ethstore::{PresaleWallet, EthStore};
use ethcore::ethstore::dir::RootDiskDirectory;
use ethcore::account_provider::{AccountProvider, AccountProviderSettings};
use helpers::{password_prompt, passwords_from_file};

use params::SpecType;

use rayon::prelude::*;

#[derive(Debug, PartialEq)]
pub struct ImportWallet {
	pub iterations: u32,
	pub path: String,
	pub spec: SpecType,
	pub wallet_path: String,
	pub password_file: Option<String>,
}

pub fn execute(cmd: ImportWallet) -> Result<String, String> {
	
	let passwords: Vec<String> = match cmd.password_file {
		Some(file) => passwords_from_file(file)?,
		None => vec![password_prompt()?],
	};
	
	println!("ImportWallet #passwords {}", passwords.len());
	let dir = Box::new(RootDiskDirectory::create(cmd.path).unwrap());
	let secret_store = Box::new(EthStore::open_with_iterations(dir, cmd.iterations).unwrap());
	let acc_provider = AccountProvider::new(secret_store, AccountProviderSettings::default());
	let wallet = PresaleWallet::open(cmd.wallet_path).map_err(|_| "Unable to open presale wallet.")?;

	let result = passwords.par_iter().find_any(|ref password| {
		if let Ok(kp) = wallet.decrypt(&password) {
			println!("WALLET DECODED, WELL DONE: password {}, key pair {:?}", password, kp);
			return true;
		}
		return false;
	});
	if let Some(password) = result {
		if let Ok(kp) = wallet.decrypt(&password) {
			let address = acc_provider.insert_account(kp.secret().clone(), &password).unwrap();
			return Ok(format!("{:?}", address))
		}
	}
	Err(format!("no correct password found"))
}
