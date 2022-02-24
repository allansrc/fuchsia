// Copyright 2021 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    anyhow::Error,
    fidl_fuchsia_bluetooth_avrcp::{PeerManagerMarker, PeerManagerRequest, TargetHandlerProxy},
    fidl_fuchsia_bluetooth_component::{LifecycleMarker, LifecycleProxy, LifecycleState},
    fidl_fuchsia_media_sessions2::{DiscoveryMarker, DiscoveryRequest, SessionsWatcherProxy},
    fidl_fuchsia_power::{BatteryManagerMarker, BatteryManagerRequest},
    fuchsia_async as fasync,
    fuchsia_component_test::new::{
        Capability, ChildOptions, LocalComponentHandles, RealmBuilder, Ref, Route,
    },
    fuchsia_zircon::DurationNum,
    futures::{channel::mpsc, SinkExt, StreamExt},
    realmbuilder_mock_helpers::mock_component,
    std::{collections::HashSet, iter::FromIterator},
    tracing::info,
};

/// AVRCP-Target component URL.
const AVRCP_TARGET_URL: &str =
    "fuchsia-pkg://fuchsia.com/bt-avrcp-target-smoke-test#meta/bt-avrcp-target.cm";

/// The different events generated by this test.
/// Note: In order to prevent channel-closure errors, the ClientEnd of the FIDL channels are
/// preserved. Each Proxy is wrapped in an Option<T> so that variants can be easily constructed
/// without the underlying FIDL channel.
enum Event {
    /// AVRCP service event.
    Avrcp(Option<TargetHandlerProxy>),
    /// Media service event.
    Media(Option<SessionsWatcherProxy>),
    /// Bluetooth Lifecycle event.
    Lifecycle(Option<LifecycleProxy>),
    /// Battery Manager service connection.
    BatteryManager(Option<BatteryManagerRequest>),
}

impl From<DiscoveryRequest> for Event {
    fn from(src: DiscoveryRequest) -> Self {
        // Only expect WatchSessions request in this integration test.
        match src {
            DiscoveryRequest::WatchSessions { session_watcher, .. } => {
                let watcher = session_watcher.into_proxy().unwrap();
                Self::Media(Some(watcher))
            }
            r => panic!("Expected Watch but got {:?}", r),
        }
    }
}

impl From<PeerManagerRequest> for Event {
    fn from(src: PeerManagerRequest) -> Self {
        // Only expect RegisterTargetHandler requests in this integration test.
        match src {
            PeerManagerRequest::RegisterTargetHandler { handler, responder, .. } => {
                let handler = handler.into_proxy().unwrap();
                responder.send(&mut Ok(())).expect("Failed to respond");
                Self::Avrcp(Some(handler))
            }
            r => panic!("Expected RegisterTargetHandler but got: {:?}", r),
        }
    }
}

impl From<BatteryManagerRequest> for Event {
    fn from(src: BatteryManagerRequest) -> Self {
        // BatteryManager requests don't need to be handled since the component-under-test is
        // resilient to unavailability.
        Self::BatteryManager(Some(src))
    }
}

/// Represents a fake AVRCP-TG client that requests the `fuchsia.bluetooth.component.Lifecycle`
/// service.
async fn mock_avrcp_target_client(
    mut sender: mpsc::Sender<Event>,
    handles: LocalComponentHandles,
) -> Result<(), Error> {
    let lifecycle_svc = handles.connect_to_protocol::<LifecycleMarker>()?;
    fasync::Task::local(async move {
        let lifecycle = lifecycle_svc.clone();
        loop {
            match lifecycle_svc.get_state().await.unwrap() {
                LifecycleState::Initializing => {}
                LifecycleState::Ready => break,
            }
            fasync::Timer::new(fasync::Time::after(1_i64.millis())).await;
        }
        info!("Client successfully connected to Lifecycle service");
        sender.send(Event::Lifecycle(Some(lifecycle))).await.expect("failed sending ack to test");
    })
    .detach();
    Ok(())
}

/// Tests that the v2 AVRCP-TG component has the correct topology and verifies that
/// it connects to the expected services.
#[fasync::run_singlethreaded(test)]
async fn avrcp_tg_v2_connects_to_avrcp_service() {
    fuchsia_syslog::init().unwrap();
    info!("Starting AVRCP-TG v2 smoke test...");

    let (sender, mut receiver) = mpsc::channel(2);
    let avrcp_tx = sender.clone();
    let media_tx = sender.clone();
    let battery_manager_tx = sender.clone();
    let fake_client_tx = sender.clone();

    let builder = RealmBuilder::new().await.expect("Failed to create test realm builder");
    // The v2 component under test.
    let avrcp_target = builder
        .add_child("avrcp-target", AVRCP_TARGET_URL.to_string(), ChildOptions::new())
        .await
        .expect("Failed adding avrcp-tg to topology");
    // Mock AVRCP component to receive PeerManager requests.
    let fake_avrcp = builder
        .add_local_child(
            "fake-avrcp",
            move |handles: LocalComponentHandles| {
                let sender = avrcp_tx.clone();
                Box::pin(mock_component::<PeerManagerMarker, _>(sender, handles))
            },
            ChildOptions::new(),
        )
        .await
        .expect("Failed adding avrcp mock to topology");
    // Mock MediaSession component to receive Discovery requests.
    let fake_media_session = builder
        .add_local_child(
            "fake-media-session",
            move |handles: LocalComponentHandles| {
                let sender = media_tx.clone();
                Box::pin(mock_component::<DiscoveryMarker, _>(sender, handles))
            },
            ChildOptions::new(),
        )
        .await
        .expect("Failed adding media session mock to topology");
    // Mock BatteryManager component to receive BatteryManager requests.
    let fake_battery_manager = builder
        .add_local_child(
            "fake-battery-manager",
            move |handles: LocalComponentHandles| {
                let sender = battery_manager_tx.clone();
                Box::pin(mock_component::<BatteryManagerMarker, _>(sender, handles))
            },
            ChildOptions::new(),
        )
        .await
        .expect("Failed adding battery manager mock to topology");
    // Mock AVRCP-Target client that will request the Lifecycle service.
    let fake_avrcp_target_client = builder
        .add_local_child(
            "fake-avrcp-target-client",
            move |handles: LocalComponentHandles| {
                let sender = fake_client_tx.clone();
                Box::pin(mock_avrcp_target_client(sender, handles))
            },
            ChildOptions::new().eager(),
        )
        .await
        .expect("Failed adding avrcp target client mock to topology");

    // Set up capabilities.
    builder
        .add_route(
            Route::new()
                .capability(Capability::protocol::<PeerManagerMarker>())
                .from(&fake_avrcp)
                .to(&avrcp_target),
        )
        .await
        .expect("Failed adding route for PeerManager service");
    builder
        .add_route(
            Route::new()
                .capability(Capability::protocol::<DiscoveryMarker>())
                .from(&fake_media_session)
                .to(&avrcp_target),
        )
        .await
        .expect("Failed adding route for Discovery service");
    builder
        .add_route(
            Route::new()
                .capability(Capability::protocol::<BatteryManagerMarker>())
                .from(&fake_battery_manager)
                .to(&avrcp_target),
        )
        .await
        .expect("Failed adding route for Discovery service");
    builder
        .add_route(
            Route::new()
                .capability(Capability::protocol::<LifecycleMarker>())
                .from(&avrcp_target)
                .to(&fake_avrcp_target_client),
        )
        .await
        .expect("Failed adding route for Lifecycle service");
    builder
        .add_route(
            Route::new()
                .capability(Capability::protocol::<fidl_fuchsia_logger::LogSinkMarker>())
                .from(Ref::parent())
                .to(&avrcp_target)
                .to(&fake_avrcp)
                .to(&fake_battery_manager)
                .to(&fake_media_session)
                .to(&fake_avrcp_target_client),
        )
        .await
        .expect("Failed adding LogSink route to test components");
    let _test_topology = builder.build().await.unwrap();

    // If the routing is correctly configured, we expect four events: `bt-avrcp-target` connecting
    // to the PeerManager, Discovery, & BatteryManager services and the fake client connecting to
    // the Lifecycle service that is provided by `bt-avrcp-target`.
    let mut events = Vec::new();
    let expected_number_of_events = 4;
    for i in 0..expected_number_of_events {
        let msg = format!("Unexpected error waiting for {:?} event", i);
        events.push(receiver.next().await.expect(&msg));
    }
    assert_eq!(events.len(), expected_number_of_events);

    let discriminants: HashSet<_> = HashSet::from_iter(events.iter().map(std::mem::discriminant));
    // Expect one request of each.
    let expected: HashSet<_> = HashSet::from_iter(
        vec![
            Event::Avrcp(None),
            Event::Media(None),
            Event::BatteryManager(None),
            Event::Lifecycle(None),
        ]
        .iter()
        .map(std::mem::discriminant),
    );
    assert_eq!(discriminants, expected);

    info!("Finished AVRCP-TG smoke test");
}