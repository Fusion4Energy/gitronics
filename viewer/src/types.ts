// The project manifest emitted by `gitronics inspect` (project_report.json).
// Keep in sync with `src/project_report.rs`.

export type MetadataValue = string | number | boolean | null | MetadataValue[] | { [k: string]: MetadataValue };
export type Metadata = Record<string, MetadataValue>;

export interface FillerLib {
  name: string;
  universe_id: number;
  cell_count: number;
  surface_count: number;
  materials: number[];
  cell_id_runs: [number, number][];
  surface_id_runs: [number, number][];
  /** `description` is recognized first-class; all other keys are project-defined. */
  description?: string;
  metadata: Metadata;
  /** Names of configurations that place this filler in at least one envelope. */
  used_by_configs: string[];
}

export interface EnvelopeInv {
  envelope_name: string;
  description?: string;
  metadata: Metadata;
}

export interface ConfigEnvelope {
  envelope_name: string;
  filler_name?: string;
  universe_id?: number;
  transform?: string;
}

export interface ConfigStats {
  filled: number;
  unfilled: number;
  distinct_fillers: number;
  unused_fillers: number;
}

export interface ConfigEntry {
  name: string;
  path: string;
  overrides?: string;
  envelopes: ConfigEnvelope[];
  stats: ConfigStats;
}

export interface MetadataKeys {
  filler: string[];
  envelope: string[];
}

export interface ProjectReport {
  schema_version: number;
  gitronics_version: string;
  commit_hash: string;
  date_time: string;
  project_dir: string;
  filler_library: FillerLib[];
  envelope_inventory: EnvelopeInv[];
  configurations: ConfigEntry[];
  metadata_keys: MetadataKeys;
}
