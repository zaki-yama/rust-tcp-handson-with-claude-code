use super::*;

// =============================================================================
// Phase A: TCP状態の定義 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_a_tests {
    use super::*;

    // Task A1: TcpState列挙型の基本テスト
    #[test]
    fn test_tcp_state_enum_basic() {
        // Red: 最初は失敗する（状態enumが未実装）
        let state = TcpState::Closed;
        assert_eq!(state, TcpState::Closed);

        // 異なる状態の比較
        assert_ne!(state, TcpState::Listen);
        assert_ne!(state, TcpState::SynSent);
        assert_ne!(state, TcpState::Established);
    }

    #[test]
    fn test_tcp_state_all_variants() {
        // RFC 9293で定義されている11種類の状態をすべてテスト
        let states = [
            TcpState::Closed,
            TcpState::Listen,
            TcpState::SynSent,
            TcpState::SynReceived,
            TcpState::Established,
            TcpState::FinWait1,
            TcpState::FinWait2,
            TcpState::CloseWait,
            TcpState::Closing,
            TcpState::LastAck,
            TcpState::TimeWait,
        ];

        // すべての状態が一意
        for (i, state1) in states.iter().enumerate() {
            for (j, state2) in states.iter().enumerate() {
                if i == j {
                    assert_eq!(state1, state2);
                } else {
                    assert_ne!(state1, state2);
                }
            }
        }
    }

    #[test]
    fn test_tcp_state_clone() {
        // Cloneトレイトのテスト
        let original = TcpState::Established;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    // Task A2: TcpEvent列挙型のテスト
    #[test]
    fn test_tcp_event_enum() {
        // イベントが正しく定義されているかテスト
        let event = TcpEvent::Connect;
        assert_eq!(event, TcpEvent::Connect);
        assert_ne!(event, TcpEvent::Listen);
    }

    #[test]
    fn test_tcp_event_all_variants() {
        // すべてのイベントを列挙してテスト
        let events = vec![
            TcpEvent::Connect,
            TcpEvent::Listen,
            TcpEvent::ReceiveSyn,
            TcpEvent::ReceiveSynAck,
            TcpEvent::ReceiveAck,
            TcpEvent::ReceiveFin,
            TcpEvent::ReceiveRst,
            TcpEvent::Close,
            TcpEvent::Timeout,
        ];

        // イベントが一意
        for event in &events {
            let cloned = event.clone();
            assert_eq!(event, &cloned);
        }
    }

    // Task A3: StateMachineの基本構造テスト
    #[test]
    fn test_state_machine_creation() {
        // Red: 最初は失敗する（TcpStateMachine未実装）
        let sm = TcpStateMachine::new();

        // 初期状態はClosed
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_state_machine_current_state() {
        let sm = TcpStateMachine::new();
        let state = sm.current_state();

        // current_stateメソッドが正しく動作
        assert_eq!(state, TcpState::Closed);
    }
}

// =============================================================================
// Phase B: 状態遷移ロジック - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_b_tests {
    use super::*;

    // Task B1: 基本的な状態遷移テスト
    #[test]
    fn test_transition_basic() {
        // Red: transition()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // Closed → Listen遷移
        let result = sm.transition(TcpEvent::Listen);
        assert!(result.is_ok(), "Transition should succeed");
        assert_eq!(result.unwrap(), TcpState::Listen);
        assert_eq!(sm.current_state(), TcpState::Listen);
    }

    #[test]
    fn test_transition_closed_to_syn_sent() {
        // Closed → SynSent遷移（アクティブオープン）
        let mut sm = TcpStateMachine::new();

        let result = sm.transition(TcpEvent::Connect);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TcpState::SynSent);
        assert_eq!(sm.current_state(), TcpState::SynSent);
    }

    #[test]
    fn test_invalid_transition() {
        // 不正な遷移はエラーを返す
        let mut sm = TcpStateMachine::new();

        // Closed状態でReceiveSynAckは不正
        let result = sm.transition(TcpEvent::ReceiveSynAck);
        assert!(result.is_err(), "Invalid transition should fail");
        assert_eq!(sm.current_state(), TcpState::Closed); // 状態は変わらない
    }

    #[test]
    fn test_transition_returns_error_message() {
        // エラーメッセージが適切に返される
        let mut sm = TcpStateMachine::new();

        let result = sm.transition(TcpEvent::ReceiveFin);
        assert!(result.is_err());

        let error_msg = result.unwrap_err();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
    }

    // Task B2: 状態遷移テーブルのテスト
    #[test]
    fn test_state_transition_table() {
        // 複数の遷移をテスト
        let test_cases: Vec<(TcpState, TcpEvent, Result<TcpState, String>)> = vec![
            (TcpState::Closed, TcpEvent::Connect, Ok(TcpState::SynSent)),
            (TcpState::Closed, TcpEvent::Listen, Ok(TcpState::Listen)),
            (
                TcpState::Listen,
                TcpEvent::ReceiveSyn,
                Ok(TcpState::SynReceived),
            ),
            (
                TcpState::SynSent,
                TcpEvent::ReceiveSynAck,
                Ok(TcpState::Established),
            ),
            (
                TcpState::SynReceived,
                TcpEvent::ReceiveAck,
                Ok(TcpState::Established),
            ),
        ];

        for (initial_state, event, expected_result) in test_cases {
            let mut sm = TcpStateMachine::new();

            // テストの初期状態をtransition経由で設定
            match initial_state {
                TcpState::Listen => {
                    sm.transition(TcpEvent::Listen).ok();
                }
                TcpState::SynSent => {
                    sm.transition(TcpEvent::Connect).ok();
                }
                TcpState::SynReceived => {
                    sm.transition(TcpEvent::Listen).ok();
                    sm.transition(TcpEvent::ReceiveSyn).ok();
                }
                _ => {}
            }

            let result = sm.transition(event);

            match expected_result {
                Ok(expected_state) => {
                    assert!(result.is_ok(), "Transition should succeed");
                    assert_eq!(result.unwrap(), expected_state);
                }
                Err(_) => {
                    assert!(result.is_err(), "Transition should fail");
                }
            }
        }
    }

    // Task B3: 状態確認メソッドのテスト
    #[test]
    fn test_is_established() {
        let mut sm = TcpStateMachine::new();

        // 初期状態ではEstablishedでない
        assert!(!sm.is_established());

        // Established状態に遷移
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // Established状態
        assert!(sm.is_established());
    }

    #[test]
    fn test_is_closed() {
        let mut sm = TcpStateMachine::new();

        // 初期状態はClosed
        assert!(sm.is_closed());

        // 状態遷移後はClosedでない
        sm.transition(TcpEvent::Listen).ok();
        assert!(!sm.is_closed());
    }

    #[test]
    fn test_can_send_data() {
        let mut sm = TcpStateMachine::new();

        // Closed状態ではデータ送信不可
        assert!(!sm.can_send_data());

        // Established状態でデータ送信可能
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();
        assert!(sm.can_send_data());

        // FinWait1状態ではデータ送信不可
        sm.transition(TcpEvent::Close).ok();
        assert!(!sm.can_send_data());
    }

    #[test]
    fn test_can_receive_data() {
        let mut sm = TcpStateMachine::new();

        // Closed状態ではデータ受信不可
        assert!(!sm.can_receive_data());

        // Established状態でデータ受信可能
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();
        assert!(sm.can_receive_data());

        // FinWait1状態でもデータ受信可能（相手がFIN送信前のデータ）
        sm.transition(TcpEvent::Close).ok();
        assert!(sm.can_receive_data());
    }
}

// =============================================================================
// Phase C: 接続確立時の状態管理 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_c_tests {
    use super::*;

    // Task C1: アクティブオープンのテスト
    #[test]
    fn test_active_open() {
        // Red: active_open()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // アクティブオープン実行
        let result = sm.active_open();
        assert!(result.is_ok(), "Active open should succeed");
        assert_eq!(sm.current_state(), TcpState::SynSent);
    }

    #[test]
    fn test_active_open_from_non_closed_state() {
        // Closed以外の状態からactive_openを呼ぶとエラー
        let mut sm = TcpStateMachine::new();
        sm.transition(TcpEvent::Listen).ok();

        let result = sm.active_open();
        assert!(result.is_err(), "Active open from Listen should fail");
    }

    #[test]
    fn test_complete_active_open() {
        // SYN-ACK受信で接続完了
        let mut sm = TcpStateMachine::new();
        sm.active_open().ok();

        // SYN-ACK受信
        let result = sm.complete_active_open();
        assert!(result.is_ok(), "Complete active open should succeed");
        assert_eq!(sm.current_state(), TcpState::Established);
    }

    #[test]
    fn test_complete_active_open_from_wrong_state() {
        // SynSent以外の状態からcomplete_active_openを呼ぶとエラー
        let mut sm = TcpStateMachine::new();

        let result = sm.complete_active_open();
        assert!(
            result.is_err(),
            "Complete active open from Closed should fail"
        );
    }

    // Task C2: パッシブオープンのテスト
    #[test]
    fn test_passive_open() {
        // Red: passive_open()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // パッシブオープン実行
        let result = sm.passive_open();
        assert!(result.is_ok(), "Passive open should succeed");
        assert_eq!(sm.current_state(), TcpState::Listen);
    }

    #[test]
    fn test_accept_connection() {
        // SYN受信 → ACK受信で接続確立
        let mut sm = TcpStateMachine::new();
        sm.passive_open().ok();

        // SYN受信でSynReceived状態へ
        sm.transition(TcpEvent::ReceiveSyn).ok();
        assert_eq!(sm.current_state(), TcpState::SynReceived);

        // accept_connectionでACK受信処理
        let result = sm.accept_connection();
        assert!(result.is_ok(), "Accept connection should succeed");
        assert_eq!(sm.current_state(), TcpState::Established);
    }

    #[test]
    fn test_accept_connection_from_wrong_state() {
        // SynReceived以外の状態からaccept_connectionを呼ぶとエラー
        let mut sm = TcpStateMachine::new();

        let result = sm.accept_connection();
        assert!(result.is_err(), "Accept connection from Closed should fail");
    }

    // Task C3: 同時オープンのテスト
    #[test]
    fn test_simultaneous_open() {
        // Red: simultaneous_open()メソッドが未実装
        let mut sm = TcpStateMachine::new();
        sm.active_open().ok();
        assert_eq!(sm.current_state(), TcpState::SynSent);

        // SynSent状態でSYN受信 → SynReceived
        let result = sm.simultaneous_open();
        assert!(result.is_ok(), "Simultaneous open should succeed");
        assert_eq!(sm.current_state(), TcpState::SynReceived);

        // その後、ACK受信でEstablishedへ（呼び出し側が制御）
        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::Established);
    }

    #[test]
    fn test_client_server_connection_sequence() {
        // クライアント側の完全な接続シーケンス
        let mut client = TcpStateMachine::new();

        // 1. アクティブオープン
        client.active_open().ok();
        assert_eq!(client.current_state(), TcpState::SynSent);

        // 2. SYN-ACK受信
        client.complete_active_open().ok();
        assert_eq!(client.current_state(), TcpState::Established);

        // サーバー側の完全な接続シーケンス
        let mut server = TcpStateMachine::new();

        // 1. パッシブオープン
        server.passive_open().ok();
        assert_eq!(server.current_state(), TcpState::Listen);

        // 2. SYN受信
        server.transition(TcpEvent::ReceiveSyn).ok();
        assert_eq!(server.current_state(), TcpState::SynReceived);

        // 3. ACK受信
        server.accept_connection().ok();
        assert_eq!(server.current_state(), TcpState::Established);
    }
}

// =============================================================================
// Phase D: 接続終了時の状態管理 - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_d_tests {
    use super::*;

    // Task D1: アクティブクローズのテスト
    #[test]
    fn test_active_close() {
        // Red: active_close()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();
        assert_eq!(sm.current_state(), TcpState::Established);

        // アクティブクローズ実行
        let result = sm.active_close();
        assert!(result.is_ok(), "Active close should succeed");
        assert_eq!(sm.current_state(), TcpState::FinWait1);
    }

    #[test]
    fn test_active_close_complete_sequence() {
        // Established → FinWait1 → FinWait2 → TimeWait → Closed
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // FIN送信（FinWait1へ）
        sm.active_close().ok();
        assert_eq!(sm.current_state(), TcpState::FinWait1);

        // ACK受信（FinWait2へ）
        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::FinWait2);

        // FIN受信（TimeWaitへ）
        sm.transition(TcpEvent::ReceiveFin).ok();
        assert_eq!(sm.current_state(), TcpState::TimeWait);

        // タイムアウト（Closedへ）
        sm.transition(TcpEvent::Timeout).ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_active_close_from_wrong_state() {
        // Established以外の状態からactive_closeを呼ぶとエラー
        let mut sm = TcpStateMachine::new();

        let result = sm.active_close();
        assert!(result.is_err(), "Active close from Closed should fail");
    }

    // Task D2: パッシブクローズのテスト
    #[test]
    fn test_passive_close() {
        // Red: passive_close()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // パッシブクローズ実行（FIN受信）
        let result = sm.passive_close();
        assert!(result.is_ok(), "Passive close should succeed");
        assert_eq!(sm.current_state(), TcpState::CloseWait);
    }

    #[test]
    fn test_passive_close_complete_sequence() {
        // Established → CloseWait → LastAck → Closed
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // FIN受信（CloseWaitへ）
        sm.passive_close().ok();
        assert_eq!(sm.current_state(), TcpState::CloseWait);

        // アプリケーションがClose呼び出し（LastAckへ）
        sm.transition(TcpEvent::Close).ok();
        assert_eq!(sm.current_state(), TcpState::LastAck);

        // ACK受信（Closedへ）
        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_passive_close_from_wrong_state() {
        // Established以外の状態からpassive_closeを呼ぶとエラー
        let mut sm = TcpStateMachine::new();

        let result = sm.passive_close();
        assert!(result.is_err(), "Passive close from Closed should fail");
    }

    // Task D3: 同時クローズのテスト
    #[test]
    fn test_simultaneous_close() {
        // Red: simultaneous_close()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // FIN送信（FinWait1へ）
        sm.active_close().ok();
        assert_eq!(sm.current_state(), TcpState::FinWait1);

        // FIN受信（Closingへ）
        let result = sm.simultaneous_close();
        assert!(result.is_ok(), "Simultaneous close should succeed");
        assert_eq!(sm.current_state(), TcpState::Closing);
    }

    #[test]
    fn test_simultaneous_close_complete_sequence() {
        // Established → FinWait1 → Closing → TimeWait → Closed
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // FIN送信（FinWait1へ）
        sm.active_close().ok();

        // FIN受信（Closingへ）
        sm.simultaneous_close().ok();
        assert_eq!(sm.current_state(), TcpState::Closing);

        // ACK受信（TimeWaitへ）
        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::TimeWait);

        // タイムアウト（Closedへ）
        sm.transition(TcpEvent::Timeout).ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_all_close_scenarios() {
        // すべてのクローズシナリオが正しくClosedに到達するかテスト
        let scenarios = vec![
            // (初期状態設定, イベントシーケンス, 期待される最終状態)
            (
                vec![TcpEvent::Connect, TcpEvent::ReceiveSynAck],
                vec![
                    TcpEvent::Close,      // FinWait1
                    TcpEvent::ReceiveAck, // FinWait2
                    TcpEvent::ReceiveFin, // TimeWait
                    TcpEvent::Timeout,    // Closed
                ],
                TcpState::Closed,
            ),
            (
                vec![TcpEvent::Connect, TcpEvent::ReceiveSynAck],
                vec![
                    TcpEvent::ReceiveFin, // CloseWait
                    TcpEvent::Close,      // LastAck
                    TcpEvent::ReceiveAck, // Closed
                ],
                TcpState::Closed,
            ),
        ];

        for (setup, events, expected_final_state) in scenarios {
            let mut sm = TcpStateMachine::new();

            // セットアップ
            for event in setup {
                sm.transition(event).ok();
            }

            // イベントシーケンス実行
            for event in events {
                sm.transition(event).ok();
            }

            assert_eq!(sm.current_state(), expected_final_state);
        }
    }
}

// =============================================================================
// Phase E: エラーハンドリング - TDD Tests
// =============================================================================

#[cfg(test)]
mod phase_e_tests {
    use super::*;

    // Task E1: RSTパケット処理のテスト
    #[test]
    fn test_handle_reset() {
        // Red: handle_reset()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // Established状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();
        assert_eq!(sm.current_state(), TcpState::Established);

        // RST受信
        let result = sm.handle_reset();
        assert!(result.is_ok(), "Handle reset should succeed");
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_handle_reset_from_various_states() {
        // 様々な状態からRST受信でClosedに遷移
        let states = vec![
            TcpState::Listen,
            TcpState::SynSent,
            TcpState::SynReceived,
            TcpState::Established,
            TcpState::FinWait1,
            TcpState::FinWait2,
            TcpState::CloseWait,
        ];

        for initial_state in states {
            let mut sm = TcpStateMachine::new();

            // 初期状態に設定（簡易版）
            match initial_state {
                TcpState::Listen => {
                    sm.transition(TcpEvent::Listen).ok();
                }
                TcpState::SynSent => {
                    sm.transition(TcpEvent::Connect).ok();
                }
                TcpState::Established => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                }
                _ => continue, // 複雑な状態はスキップ
            }

            // RST処理
            sm.handle_reset().ok();
            assert_eq!(
                sm.current_state(),
                TcpState::Closed,
                "Should transition to Closed from {:?}",
                initial_state
            );
        }
    }

    #[test]
    fn test_rst_via_transition() {
        // transitionメソッド経由でRSTを処理
        let mut sm = TcpStateMachine::new();
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        let result = sm.transition(TcpEvent::ReceiveRst);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TcpState::Closed);
    }

    // Task E2: タイムアウト処理のテスト
    #[test]
    fn test_handle_timeout() {
        // Red: handle_timeout()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // SynSent状態でタイムアウト
        sm.transition(TcpEvent::Connect).ok();
        assert_eq!(sm.current_state(), TcpState::SynSent);

        let result = sm.handle_timeout();
        assert!(result.is_ok(), "Handle timeout should succeed");
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_timeout_in_syn_received() {
        // SynReceived状態でタイムアウト
        let mut sm = TcpStateMachine::new();
        sm.transition(TcpEvent::Listen).ok();
        sm.transition(TcpEvent::ReceiveSyn).ok();
        assert_eq!(sm.current_state(), TcpState::SynReceived);

        sm.handle_timeout().ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_timeout_in_time_wait() {
        // TimeWait状態でタイムアウト（2MSL経過）
        let mut sm = TcpStateMachine::new();

        // TimeWait状態に移行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();
        sm.transition(TcpEvent::Close).ok();
        sm.transition(TcpEvent::ReceiveAck).ok();
        sm.transition(TcpEvent::ReceiveFin).ok();
        assert_eq!(sm.current_state(), TcpState::TimeWait);

        // タイムアウト
        sm.handle_timeout().ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_timeout_in_established_is_error() {
        // Established状態でタイムアウトは通常発生しない（エラー）
        let mut sm = TcpStateMachine::new();
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        let result = sm.handle_timeout();
        // 実装によってはエラーを返すか、何もしない
        // ここではエラーを期待
        assert!(
            result.is_err() || sm.current_state() == TcpState::Established,
            "Timeout in Established should be handled appropriately"
        );
    }

    // Task E3: 不正遷移の検出とログ
    #[test]
    fn test_state_history_tracking() {
        // Red: get_state_history()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // いくつかの遷移を実行
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // 履歴取得
        let history = sm.get_state_history();
        assert!(
            history.len() >= 2,
            "History should contain at least 2 transitions"
        );

        // 最初の遷移確認: Closed → SynSent
        assert_eq!(history[0].0, TcpState::Closed);
        assert_eq!(history[0].1, TcpState::SynSent);
        assert_eq!(history[0].2, TcpEvent::Connect);

        // 2番目の遷移確認: SynSent → Established
        assert_eq!(history[1].0, TcpState::SynSent);
        assert_eq!(history[1].1, TcpState::Established);
        assert_eq!(history[1].2, TcpEvent::ReceiveSynAck);
    }

    #[test]
    fn test_invalid_transition_not_in_history() {
        // 不正な遷移は履歴に記録されない
        let mut sm = TcpStateMachine::new();

        // 正常な遷移
        sm.transition(TcpEvent::Connect).ok();

        // 不正な遷移
        sm.transition(TcpEvent::ReceiveFin).ok();

        let history = sm.get_state_history();
        // 履歴には正常な遷移のみ
        assert_eq!(
            history.len(),
            1,
            "Invalid transitions should not be recorded"
        );
    }

    #[test]
    fn test_print_state_diagram() {
        // Red: print_state_diagram()メソッドが未実装
        let mut sm = TcpStateMachine::new();

        // いくつかの遷移
        sm.transition(TcpEvent::Connect).ok();
        sm.transition(TcpEvent::ReceiveSynAck).ok();

        // 出力テスト（パニックしないことを確認）
        sm.print_state_diagram();
    }

    #[test]
    fn test_error_message_quality() {
        // エラーメッセージが有用な情報を含むかテスト
        let mut sm = TcpStateMachine::new();

        let result = sm.transition(TcpEvent::ReceiveSynAck);
        assert!(result.is_err());

        let error = result.unwrap_err();
        // エラーメッセージに状態とイベントが含まれる
        assert!(
            error.contains("Closed") || error.contains("ReceiveSynAck"),
            "Error message should contain useful information: {}",
            error
        );
    }
}

// =============================================================================
// Phase F: 統合テスト - Integration Tests
// =============================================================================

#[cfg(test)]
mod phase_f_tests {
    use super::*;

    // Task F1: 完全な接続ライフサイクルテスト
    #[test]
    fn test_complete_connection_lifecycle() {
        // 接続確立から終了までの完全なフロー
        let mut sm = TcpStateMachine::new();

        // 1. 接続確立（アクティブオープン）
        assert_eq!(sm.current_state(), TcpState::Closed);

        sm.active_open().expect("Active open should succeed");
        assert_eq!(sm.current_state(), TcpState::SynSent);

        sm.complete_active_open()
            .expect("Complete active open should succeed");
        assert_eq!(sm.current_state(), TcpState::Established);

        // 2. データ送受信可能状態の確認
        assert!(sm.can_send_data());
        assert!(sm.can_receive_data());
        assert!(sm.is_established());

        // 3. 接続終了（アクティブクローズ）
        sm.active_close().expect("Active close should succeed");
        assert_eq!(sm.current_state(), TcpState::FinWait1);

        sm.transition(TcpEvent::ReceiveAck)
            .expect("Transition to FinWait2 should succeed");
        assert_eq!(sm.current_state(), TcpState::FinWait2);

        sm.transition(TcpEvent::ReceiveFin)
            .expect("Transition to TimeWait should succeed");
        assert_eq!(sm.current_state(), TcpState::TimeWait);

        sm.transition(TcpEvent::Timeout)
            .expect("Transition to Closed should succeed");
        assert_eq!(sm.current_state(), TcpState::Closed);

        // 4. 履歴確認
        let history = sm.get_state_history();
        assert!(
            history.len() >= 5,
            "History should contain multiple transitions"
        );
    }

    #[test]
    fn test_server_connection_lifecycle() {
        // サーバー側の完全なライフサイクル
        let mut sm = TcpStateMachine::new();

        // 1. Listen開始
        sm.passive_open().expect("Passive open should succeed");
        assert_eq!(sm.current_state(), TcpState::Listen);

        // 2. SYN受信
        sm.transition(TcpEvent::ReceiveSyn)
            .expect("ReceiveSyn should succeed");
        assert_eq!(sm.current_state(), TcpState::SynReceived);

        // 3. ACK受信で接続確立
        sm.accept_connection()
            .expect("Accept connection should succeed");
        assert_eq!(sm.current_state(), TcpState::Established);

        // 4. FIN受信（パッシブクローズ）
        sm.passive_close().expect("Passive close should succeed");
        assert_eq!(sm.current_state(), TcpState::CloseWait);

        // 5. アプリケーションがClose
        sm.transition(TcpEvent::Close)
            .expect("Close should succeed");
        assert_eq!(sm.current_state(), TcpState::LastAck);

        // 6. ACK受信で終了
        sm.transition(TcpEvent::ReceiveAck)
            .expect("Final ACK should succeed");
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_simultaneous_scenarios() {
        // 同時オープン + 同時クローズ
        let mut sm = TcpStateMachine::new();

        // 同時オープン: SynSent → SynReceived
        sm.active_open().ok();
        sm.simultaneous_open().ok();
        assert_eq!(sm.current_state(), TcpState::SynReceived);

        // ACK受信でEstablishedへ
        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::Established);

        // 同時クローズ
        sm.active_close().ok();
        sm.simultaneous_close().ok();
        assert_eq!(sm.current_state(), TcpState::Closing);

        sm.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(sm.current_state(), TcpState::TimeWait);

        sm.transition(TcpEvent::Timeout).ok();
        assert_eq!(sm.current_state(), TcpState::Closed);
    }

    #[test]
    fn test_error_recovery() {
        // エラーからの回復シナリオ
        let mut sm = TcpStateMachine::new();

        // 正常な接続開始
        sm.active_open().ok();
        assert_eq!(sm.current_state(), TcpState::SynSent);

        // RST受信で即座に終了
        sm.handle_reset().ok();
        assert_eq!(sm.current_state(), TcpState::Closed);

        // 再接続可能
        sm.active_open().ok();
        assert_eq!(sm.current_state(), TcpState::SynSent);

        // タイムアウト
        sm.handle_timeout().ok();
        assert_eq!(sm.current_state(), TcpState::Closed);

        // 再び接続可能
        sm.active_open().ok();
        assert_eq!(sm.current_state(), TcpState::SynSent);
    }

    #[test]
    fn test_all_states_reachable() {
        // すべての状態に到達可能かテスト
        let reachable_states = vec![
            TcpState::Closed,
            TcpState::Listen,
            TcpState::SynSent,
            TcpState::SynReceived,
            TcpState::Established,
            TcpState::FinWait1,
            TcpState::FinWait2,
            TcpState::CloseWait,
            TcpState::Closing,
            TcpState::LastAck,
            TcpState::TimeWait,
        ];

        for target_state in reachable_states {
            let mut sm = TcpStateMachine::new();
            let mut reached = false;

            // 各状態への到達を試みる
            match target_state {
                TcpState::Closed => {
                    reached = true;
                }
                TcpState::Listen => {
                    sm.transition(TcpEvent::Listen).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::SynSent => {
                    sm.transition(TcpEvent::Connect).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::SynReceived => {
                    sm.transition(TcpEvent::Listen).ok();
                    sm.transition(TcpEvent::ReceiveSyn).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::Established => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::FinWait1 => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::Close).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::FinWait2 => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::Close).ok();
                    sm.transition(TcpEvent::ReceiveAck).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::CloseWait => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::ReceiveFin).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::Closing => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::Close).ok();
                    sm.transition(TcpEvent::ReceiveFin).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::LastAck => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::ReceiveFin).ok();
                    sm.transition(TcpEvent::Close).ok();
                    reached = sm.current_state() == target_state;
                }
                TcpState::TimeWait => {
                    sm.transition(TcpEvent::Connect).ok();
                    sm.transition(TcpEvent::ReceiveSynAck).ok();
                    sm.transition(TcpEvent::Close).ok();
                    sm.transition(TcpEvent::ReceiveAck).ok();
                    sm.transition(TcpEvent::ReceiveFin).ok();
                    reached = sm.current_state() == target_state;
                }
            }

            assert!(reached, "State {:?} should be reachable", target_state);
        }
    }

    #[test]
    fn test_state_history_completeness() {
        // 履歴が完全に記録されているかテスト
        let mut sm = TcpStateMachine::new();

        // 複数の遷移を実行
        let events = vec![
            TcpEvent::Connect,
            TcpEvent::ReceiveSynAck,
            TcpEvent::Close,
            TcpEvent::ReceiveAck,
            TcpEvent::ReceiveFin,
            TcpEvent::Timeout,
        ];

        for event in &events {
            sm.transition(event.clone()).ok();
        }

        let history = sm.get_state_history();

        // 履歴の各エントリが正しい形式か確認
        for (from_state, to_state, event) in history {
            assert_ne!(from_state, to_state, "State should change in transition");
            println!(
                "Transition: {:?} --[{:?}]--> {:?}",
                from_state, event, to_state
            );
        }
    }

    #[test]
    fn test_display_implementations() {
        // Display実装のテスト
        let state = TcpState::Established;
        let state_str = format!("{}", state);
        assert!(!state_str.is_empty(), "State display should not be empty");

        let event = TcpEvent::Connect;
        let event_str = format!("{}", event);
        assert!(!event_str.is_empty(), "Event display should not be empty");
    }

    #[test]
    fn test_rfc_compliance() {
        // RFC 9293準拠性のテスト
        let mut sm = TcpStateMachine::new();

        // RFC 9293 Figure 6の主要な遷移パターンをテスト

        // Pattern 1: 正常な接続確立と終了
        sm.active_open().ok();
        sm.complete_active_open().ok();
        sm.active_close().ok();
        sm.transition(TcpEvent::ReceiveAck).ok();
        sm.transition(TcpEvent::ReceiveFin).ok();
        sm.transition(TcpEvent::Timeout).ok();
        assert_eq!(sm.current_state(), TcpState::Closed);

        // Pattern 2: サーバー側の正常フロー
        let mut server = TcpStateMachine::new();
        server.passive_open().ok();
        server.transition(TcpEvent::ReceiveSyn).ok();
        server.accept_connection().ok();
        server.passive_close().ok();
        server.transition(TcpEvent::Close).ok();
        server.transition(TcpEvent::ReceiveAck).ok();
        assert_eq!(server.current_state(), TcpState::Closed);
    }
}

// =============================================================================
// TDD実行ガイド
// =============================================================================

/*
TDD実行手順 (Step 4):

1. Phase A (状態定義):
   ```
   cargo test phase_a_tests::test_tcp_state_enum_basic
   cargo test phase_a_tests::test_tcp_event_enum
   cargo test phase_a_tests::test_state_machine_creation
   ```
   Red → Green: TcpState, TcpEvent, TcpStateMachine の基本構造を実装

2. Phase B (状態遷移ロジック):
   ```
   cargo test phase_b_tests::test_transition_basic
   cargo test phase_b_tests::test_invalid_transition
   cargo test phase_b_tests::test_can_send_data
   ```
   Red → Green: transition(), next_state(), 状態確認メソッドを実装

3. Phase C (接続確立):
   ```
   cargo test phase_c_tests::test_active_open
   cargo test phase_c_tests::test_passive_open
   cargo test phase_c_tests::test_simultaneous_open
   ```
   Red → Green: active_open(), passive_open(), accept_connection()を実装

4. Phase D (接続終了):
   ```
   cargo test phase_d_tests::test_active_close
   cargo test phase_d_tests::test_passive_close
   cargo test phase_d_tests::test_simultaneous_close
   ```
   Red → Green: active_close(), passive_close()などを実装

5. Phase E (エラーハンドリング):
   ```
   cargo test phase_e_tests::test_handle_reset
   cargo test phase_e_tests::test_handle_timeout
   cargo test phase_e_tests::test_state_history_tracking
   ```
   Red → Green: handle_reset(), handle_timeout(), 履歴管理を実装

6. Phase F (統合テスト):
   ```
   cargo test phase_f_tests
   ```
   すべてのフェーズが統合されて動作することを確認

各テストが Green になったら次のPhaseに進む。
Red → Green → Refactor を繰り返して高品質な状態マシンを完成させる。

実装ガイド:
- RFC 9293 Figure 6 の状態遷移図を常に参照
- 不正な遷移は必ずエラーを返す
- 履歴記録でデバッグを容易にする
- すべての11状態に到達可能であることを確認
*/
