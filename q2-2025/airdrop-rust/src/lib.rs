mod programs;

#[cfg(test)]
mod tests {
    use solana_sdk;
    use solana_sdk::{message::Message,signature::{Keypair,Signer, read_keypair_file}, transaction::Transaction, hash::hash};
    use std::io::{self, BufRead};
    use bs58;
    use solana_client::rpc_client::RpcClient;
    use solana_program::{pubkey::Pubkey, system_instruction::transfer, system_program};
    use std::str::FromStr;
    use crate::programs::Turbin3_prereq::{Turbin3PrereqProgram, CompleteArgs, UpdateArgs};
	
    const RPC_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn keygen() {
	// Create a new keypair
	let kp = Keypair::new();
	println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
	println!("");
	println!("To save your wallet copy and paste the following into a JSON file:");
	println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {
	// Import our keypair
	let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

	// Connected to Solana Devnet RPC Client
	let client = RpcClient::new(RPC_URL);

	// We're going to claim 2 devnet SOL tokens (2 billion lamports)
	match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
		Ok(s) => {
		  println!("Success! Check out your TX here:");
		  println!("https://explorer.solana.com/tx/{}?cluster=devnet", s.to_string());
		}
		Err(e) => println!("Oops, something went wrong: {}", e.to_string())
	};

    }

    #[test]
    fn transfer_sol() {
	// Import our keypair
	let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

	// With the imported Keypair, we can sign a new message
	let pubkey = keypair.pubkey();
	let message_bytes = b"I verify my solana Keypair!";
	let sig = keypair.sign_message(message_bytes);
	let sig_hashed = hash(sig.as_ref());

	// After that we can verify the signature, using the default implementation
	match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
	  true => println!("Signature verified"),
	  false => println!("Verification failed"),
	}

	// Define our Turbin3 public key
	let to_pubkey = Pubkey::from_str("ArCugaYbHumHTiwP9ArA5L2vHNgWrcVPuGSchYXhh9is").unwrap();

	// Create a Solana devnet connection
	let rpc_client = RpcClient::new(RPC_URL);

	// Get recent blockhash
	let recent_blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

	// let transaction = Transaction::new_signed_with_payer(&[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)], Some(&keypair.pubkey()), &vec![&keypair], recent_blockhash);

	// Get balance of dev wallet
	let balance = rpc_client.get_balance(&keypair.pubkey()).expect("Failed to get balance");

	// Create a test transaction to calculate fees
	let message = Message::new_with_blockhash(&[transfer(&keypair.pubkey(),&to_pubkey,balance)],Some(&keypair.pubkey()), &recent_blockhash);

	// Calculate exact fee rate to transfer entire SOL amount out of account minus fees
	let fee = rpc_client.get_fee_for_message(&message).expect("Failed to get fee calculator");

	// Deduct fee from lamports amount and create a TX with correct balance
	let transaction = Transaction::new_signed_with_payer(&[transfer(&keypair.pubkey(), &to_pubkey, balance-fee)], Some(&keypair.pubkey()), &vec![&keypair], recent_blockhash);


	// Send the transaction
	let signature = rpc_client
		.send_and_confirm_transaction(&transaction)
		.expect("Failed to send transaction");

	// Print our transaction out
	println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }


    #[test]
    fn enroll(){
	// Create a Solana devnet connection
	let rpc_client = RpcClient::new(RPC_URL);

	// Let's define our accounts
	let signer = read_keypair_file("Turbin3-wallet.json").expect("Couldn't find wallet file");

	let prereq = Turbin3PrereqProgram::derive_program_address(&[b"prereq", signer.pubkey().to_bytes().as_ref()]);

	// Define our instruction data
	// I registered once before so I need to use UpdateArgs instead
	// let args = CompleteArgs{github: b"nissan".to_vec()};
	let args = UpdateArgs{github: b"nissan".to_vec()};
	// Get recent blockhash
	let blockhash = rpc_client.get_latest_blockhash().expect("Failed to get recent blockhash");

	// Now we can invoke the "complete" function
	//let transaction = Turbin3PrereqProgram::complete(&[&signer.pubkey(), &prereq, &system_program::id()], &args, Some(&signer.pubkey()), &[&signer], blockhash);
	let transaction = Turbin3PrereqProgram::update(&[&signer.pubkey(), &prereq, &system_program::id()], &args, Some(&signer.pubkey()), &[&signer], blockhash);
	let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("Failed to send transaction");

	// Print our transaction out
	println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
    }

    #[test]
    fn base58_to_wallet(){
	println!("Input your private key as base58:");
	let stdin = io::stdin();
	let base58 = stdin.lock().lines().next().unwrap().unwrap();
	let wallet = bs58::decode(base58).into_vec().unwrap();
	println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58(){
	println!("Input your private key as a wallet file byte array:");
	let stdin = io::stdin();
	let wallet = stdin
		.lock()
		.lines()
		.next()
		.unwrap()
		.unwrap()
		.trim_start_matches('[')
		.trim_end_matches(']')
		.split(',')
		.map(|s| s.trim().parse::<u8>().unwrap())
		.collect::<Vec<u8>>();
	println!("Your private key is:");
	let base58 = bs58::encode(wallet).into_string();
	println!("{:?}", base58);
    }


}
