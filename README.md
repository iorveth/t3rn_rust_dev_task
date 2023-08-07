1. Build pallet -> `cargo build`
2. Run the test script -> `cargo test`
3. Run the test script with the logging enabled `RUST_LOG=info cargo t -- --nocapture`

    Side effect parameters bytes --> [
        [
            2,
        ],
        [
            1,
        ],
        [
            7,
        ],
        [
            92,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            60,
            85,
            23,
            125,
            103,
            176,
            100,
            187,
            93,
            24,
            154,
            62,
            29,
            218,
            217,
            188,
            102,
            70,
            224,
            46,
            100,
            214,
            227,
            8,
            245,
            172,
            187,
            21,
            51,
            172,
            67,
            13,
        ],
        [
            0,
            0,
            0,
            0,
            0,
            7,
            161,
            32,
        ],
        [
            1,
        ],
    ]

    Side effect --> MultiChain {
        chain_from: Kusama,
        chain_to: Polkadot,
        side_effect: SideEffect {
            execution_type: MultiTran {
                caller: 5c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5E9mYH6j...),
                to: 3c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d (5DRp1QQY...),
                amount: 500000,
                asset: Asset(
                    Polkadot,
                ),
            },
            state: Submitted,
        },
    }
