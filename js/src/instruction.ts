import {
	PublicKey,
	Transaction,
	TransactionInstruction,
} from "@solana/web3.js";

enum AccountType {
	OWNER = 0,
	ACCESS = 1,
}

export const initUserAccountIx = async(
	payer: PublicKey,
	authority: PublicKey,
	userAccount: PublicKey,
	systemProgram: PublicKey,
	accountType: AccountType,
): Promise<TransactionInstruction> => {
	const dataNexusProgramId = new PublicKey(DATANEXUS_PROGRAM_ID);
	const initUserAccountIx = new TransactionInstruction({
		programId: dataNexusProgramId,
		keys: [
			{ pubkey: payer, isSigner: true, isWritable: true },
			{ pubkey: authority, isSigner: false, isWritable: true },
			{ pubkey: userAccount, isSigner: false, isWritable: true },
			{ pubkey: systemProgram, isSigner: false, isWritable: false },
		],
		data: Buffer.from(Uint8Array.of(0, accountType)),
	});

	return initUserAccountIx;
};

export const initDataAccountIx = async(
	authority: PublicKey,
	ownerAccount: PublicKey,
	datasetAccount: PublicKey,
	systemProgram: PublicKey,
	hash: Uint8Array,
): Promise<TransactionInstruction> => {
	const dataNexusProgramId = new PublicKey(DATANEXUS_PROGRAM_ID);
	const initDataAccountIx = new TransactionInstruction({
		programId: dataNexusProgramId,
		keys: [
			{ pubkey: authority, isSigner: true, isWritable: true },
			{ pubkey: ownerAccount, isSigner: false, isWritable: true },
			{ pubkey: datasetAccount, isSigner: false, isWritable: true },
			{ pubkey: systemProgram, isSigner: false, isWritable: false },
		],
		data: Buffer.from(Uint8Array.of(1, ...hash)),
	});

	return initDataAccountIx;
};

export const setDataParams = async(
	authority: PublicKey,
	dataSetAccount: PublicKey,
	hash: Uint8Array,
	params: Params,
): Promise<TransactionInstruction> => {
	const dataNexusProgramId = new PublicKey(DATANEXUS_PROGRAM_ID);
	const setDataParamsIx = new TransactionInstruction({
		programId: dataNexusProgramId,
		keys: [
			{ pubkey: authority, isSigner: true, isWritable: true },
			{ pubkey: datasetAccount, isSigner: false, isWritable: true },
		],
		data: Buffer.from(Uint8Array.of(2, ...hash)),
	});

	return setDataParamsIx;
};

export const purchaseAccess = async(
	userAuthority: PublicKey,
	userAccessAccount: PublicKey,
	userTokenAccount: PublicKey,
	ownerAuthority: PublicKey,
	ownerTokenAccount: PublicKey,
	datasetAccount: PublicKey,
	tokenProgram: PublicKey,
	hash: Uint8Array,
	amount: number,
): Promise<TransactionInstruction> => {
	const dataNexusProgramId = new PublicKey(DATANEXUS_PROGRAM_ID);
	const purchaseAccessIx = new TransactionInstruction({
		programId: dataNexusProgramId,
		keys: [
			{ pubkey: userAuthority, isSigner: true, isWritable: true },
			{ pubkey: userAccessAccount, isSigner: false, isWritable: true },
			{ pubkey: userTokenAccount, isSigner: false, isWritable: true },
			{ pubkey: ownerAuthority, isSigner: false, isWritable: true },
			{ pubkey: ownerTokenAccount, isSigner: false, isWritable: true },
			{ pubkey: datasetAccount, isSigner: false, isWritable: false },
			{ pubkey: tokenProgram, isSigner: false, isWritable: false },
		],
		data: Buffer.from(Uint8Array.of(3, ...hash, ...amount))
	});

	return purchaseAccessIx;
};

export const shareAccess = async(
	userAuthority: PublicKey,
	userAccessAccount: PublicKey,
	recipientAuthority: PublicKey,
	recipientAccessAccount: PublicKey,
	datasetAccount: PublicKey,
	hash: Uint8Array,
): Promise<TransactionInstruction> => {
	const dataNexusProgramId = new PublicKey(DATANEXUS_PROGRAM_ID);
	const shareAccessIx = new TransactionInstruction({
		programId: dataNexusProgramId,
		keys: [
			{ pubkey: userAuthority, isSigner: true, isWritable: true },
			{ pubkey: userAccessAccount, isSigner: false, isWritable: true },
			{ pubkey: recipientAuthority, isSigner: false, isWritable: true },
			{ pubkey: recipientAccessAccount, isSigner: false, isWritable: true },
			{ pubkey: datasetAccount, isSigner: false, isWritable: false },
		],
		data: Buffer.from(Uint8Array.of(4, ...hash)),
	});

	return shareAccessIx;
};
