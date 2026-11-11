use flatbuffers::{FlatBufferBuilder, UnionWIPOffset, WIPOffset};
use proto::*;

fn finish(
    mut fbb: FlatBufferBuilder<'_>,
    body_type: Body,
    body: WIPOffset<UnionWIPOffset>,
) -> Vec<u8> {
    let envelope = Envelope::create(
        &mut fbb,
        &EnvelopeArgs {
            body_type,
            body: Some(body),
        },
    );
    finish_envelope_buffer(&mut fbb, envelope);
    fbb.finished_data().to_vec()
}

fn red() -> Color {
    Color::new(255, 0, 0, 255)
}

fn green() -> Color {
    Color::new(0, 255, 0, 128)
}

fn point_cloud_update(update_type: Option<UpdateType>) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::new();
    let topic_id = fbb.create_string("lidar");
    let frame_id = fbb.create_string("laser");
    let points = fbb.create_vector(&[
        PointCloudVertex::new(&FVec3::new(1.0, 2.0, 3.0), &red()),
        PointCloudVertex::new(&FVec3::new(-1.5, 0.0, 0.25), &green()),
    ]);
    let mut args = PointCloudArgs {
        ts: Some(&Time::new(10, 500)),
        topic_id: Some(topic_id),
        frame_id: Some(frame_id),
        points: Some(points),
        ..Default::default()
    };
    if let Some(update_type) = update_type {
        args.update_type = update_type;
    }
    let point_cloud = PointCloud::create(&mut fbb, &args);
    let point_clouds = fbb.create_vector(&[point_cloud]);

    let parent = fbb.create_string("base_link");
    let child = fbb.create_string("laser");
    let transform = Transform::create(
        &mut fbb,
        &TransformArgs {
            ts: None, // static
            parent_frame_id: Some(parent),
            child_frame_id: Some(child),
            rotation: Some(&DQuat::new(0.0, 0.0, 0.0, 1.0)),
            translation: Some(&DVec3::new(0.5, 0.0, 1.25)),
        },
    );
    let transforms = fbb.create_vector(&[transform]);

    let update = Update::create(
        &mut fbb,
        &UpdateArgs {
            transforms: Some(transforms),
            point_clouds: Some(point_clouds),
            ..Default::default()
        },
    );
    finish(fbb, Body::Update, update.as_union_value())
}

#[test]
fn update_roundtrips_point_cloud_and_static_transform() {
    let bytes = point_cloud_update(Some(UpdateType::Incremental));
    let envelope = root_as_envelope(&bytes).unwrap();
    assert_eq!(envelope.body_type(), Body::Update);
    assert!(envelope.body_as_metadata().is_none());
    let update = envelope.body_as_update().unwrap();

    let point_cloud = update.point_clouds().unwrap().get(0);
    assert_eq!(point_cloud.ts(), &Time::new(10, 500));
    assert_eq!(point_cloud.topic_id(), "lidar");
    assert_eq!(point_cloud.frame_id(), "laser");
    assert_eq!(point_cloud.update_type(), UpdateType::Incremental);
    let points = point_cloud.points().unwrap();
    assert_eq!(points.len(), 2);
    assert_eq!(points.get(0).position(), &FVec3::new(1.0, 2.0, 3.0));
    assert_eq!(points.get(0).color(), &red());
    assert_eq!(points.get(1).position(), &FVec3::new(-1.5, 0.0, 0.25));
    assert_eq!(points.get(1).color(), &green());

    let transform = update.transforms().unwrap().get(0);
    assert!(transform.ts().is_none());
    assert_eq!(transform.parent_frame_id(), "base_link");
    assert_eq!(transform.child_frame_id(), "laser");
    assert_eq!(transform.rotation(), &DQuat::new(0.0, 0.0, 0.0, 1.0));
    assert_eq!(transform.translation(), &DVec3::new(0.5, 0.0, 1.25));
}

#[test]
fn metadata_roundtrips_static_mesh_and_open_ended_log() {
    let mut fbb = FlatBufferBuilder::new();
    let mesh_id = fbb.create_string("cube");
    let vertices = fbb.create_vector(&[
        MeshVertex::new(&FVec3::new(0.0, 0.0, 0.0), &red()),
        MeshVertex::new(&FVec3::new(1.0, 0.0, 0.0), &green()),
        MeshVertex::new(&FVec3::new(0.0, 1.0, 0.0), &red()),
    ]);
    let indices = fbb.create_vector(&[0u32, 1, 2]);
    let mesh = StaticMesh::create(
        &mut fbb,
        &StaticMeshArgs {
            mesh_id: Some(mesh_id),
            vertices: Some(vertices),
            indices: Some(indices),
        },
    );
    let meshes = fbb.create_vector(&[mesh]);
    let log = LogMetadata::create(
        &mut fbb,
        &LogMetadataArgs {
            start: Some(&Time::new(100, 0)),
            end: None, // still recording
        },
    );
    let metadata = Metadata::create(
        &mut fbb,
        &MetadataArgs {
            log: Some(log),
            meshes: Some(meshes),
        },
    );
    let bytes = finish(fbb, Body::Metadata, metadata.as_union_value());

    let envelope = root_as_envelope(&bytes).unwrap();
    assert_eq!(envelope.body_type(), Body::Metadata);
    let metadata = envelope.body_as_metadata().unwrap();

    let log = metadata.log().unwrap();
    assert_eq!(log.start(), Some(&Time::new(100, 0)));
    assert!(log.end().is_none());

    let mesh = metadata.meshes().unwrap().get(0);
    assert_eq!(mesh.mesh_id(), "cube");
    let vertices = mesh.vertices().unwrap();
    assert_eq!(vertices.len(), 3);
    assert_eq!(vertices.get(1).position(), &FVec3::new(1.0, 0.0, 0.0));
    assert_eq!(vertices.get(1).color(), &green());
    let indices: Vec<u32> = mesh.indices().unwrap().iter().collect();
    assert_eq!(indices, [0, 1, 2]);
}

#[test]
fn absent_update_type_reads_as_snapshot() {
    let bytes = point_cloud_update(None);
    let update = root_as_envelope(&bytes).unwrap().body_as_update().unwrap();
    let point_cloud = update.point_clouds().unwrap().get(0);
    assert_eq!(point_cloud.update_type(), UpdateType::Snapshot);
}

#[test]
fn absent_update_vectors_read_as_none() {
    let mut fbb = FlatBufferBuilder::new();
    let update = Update::create(&mut fbb, &UpdateArgs::default());
    let bytes = finish(fbb, Body::Update, update.as_union_value());

    let update = root_as_envelope(&bytes).unwrap().body_as_update().unwrap();
    assert!(update.transforms().is_none());
    assert!(update.poses().is_none());
    assert!(update.point_clouds().is_none());
    assert!(update.dynamic_meshes().is_none());
    assert!(update.static_meshes().is_none());
    assert!(update.paths().is_none());
}

#[test]
fn truncated_buffer_fails_verification() {
    let bytes = point_cloud_update(Some(UpdateType::Incremental));
    assert!(root_as_envelope(&bytes).is_ok());
    // not `len - 1`: the verifier only checks referenced bytes, and the builder may end
    // the buffer with unreferenced padding
    for len in [0, 4, 8, bytes.len() / 2] {
        assert!(root_as_envelope(&bytes[..len]).is_err(), "len {len}");
    }
}

#[test]
fn identifier_check_rejects_altered_identifier() {
    let mut bytes = point_cloud_update(Some(UpdateType::Incremental));
    assert!(envelope_buffer_has_identifier(&bytes));
    // the identifier sits right after the 4-byte root offset
    assert_eq!(&bytes[4..8], b"NAVI");
    bytes[4..8].copy_from_slice(b"XXXX");
    assert!(!envelope_buffer_has_identifier(&bytes));
}
