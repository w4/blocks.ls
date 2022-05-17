CREATE TABLE blocks (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    hash BYTEA NOT NULL,
    height BIGINT NOT NULL,
    version INT NOT NULL,
    size INT NOT NULL,
    previous_block_id BIGINT,
    merkle_root_hash BYTEA NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    bits INT NOT NULL,
    nonce INT NOT NULL,
    difficulty BIGINT NOT NULL,
    CONSTRAINT fk_previous_block_id
        FOREIGN KEY(previous_block_id)
            REFERENCES blocks(id)
);

CREATE TABLE transactions (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    hash BYTEA UNIQUE NOT NULL,
    block_id BIGINT,
    version INT NOT NULL,
    lock_time INT NOT NULL,
    weight BIGINT NOT NULL,
    coinbase BOOLEAN NOT NULL,
    replace_by_fee BOOLEAN NOT NULL,
    CONSTRAINT fk_block_id
        FOREIGN KEY(block_id)
            REFERENCES blocks(id)
);

CREATE TABLE transaction_outputs (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    transaction_id BIGINT NOT NULL,
    index BIGINT NOT NULL,
    value BIGINT NOT NULL,
    script BYTEA NOT NULL,
    unspendable BOOLEAN NOT NULL,
    address VARCHAR,
    CONSTRAINT fk_transaction_id
        FOREIGN KEY(transaction_id)
            REFERENCES transactions(id)
);

CREATE INDEX transaction_outputs_txid_index ON transaction_outputs (transaction_id, index);

CREATE TABLE transaction_inputs (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    transaction_id BIGINT NOT NULL,
    previous_output BIGINT,
    script BYTEA NOT NULL,
    address VARCHAR,
    CONSTRAINT fk_transaction_id
        FOREIGN KEY(transaction_id)
            REFERENCES transactions,
    CONSTRAINT fk_previous_output
        FOREIGN KEY(previous_output)
            REFERENCES transaction_outputs(id)
);
