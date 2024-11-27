use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Blueprint {
  item: String,
  label: String,
  version: i64,
  entities: Vec<BlueprintEntity>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlueprintEntity {
  entity_number: i32,
  name: String,
  position: Position<f32>,
  connections: Connection,
  control_behavior: ControlBehavior,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Position<T> {
  x: T,
  y: T,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Connection {
  #[serde(rename = "1")]
  _1: ConnectionPoint,
  #[serde(rename = "2")]
  _2: ConnectionPoint
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectionPoint {
  red: Vec<ConnectionData>,
  green: Vec<ConnectionData>
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectionData {
  entity_id: i32,
  circuit_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ControlBehavior {
  //todo
}

