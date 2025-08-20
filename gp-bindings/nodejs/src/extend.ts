import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import path from "path";
import { decodeZones } from "./utils";

// Needed for ESM builds to get __dirname
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const require = createRequire(import.meta.url);
// Dynamically load the addon binary
// This will resolve to dist/index.node after build

const { Dggrs: Aux } = require(path.join(__dirname, "index.node"));

export class Dggrs extends Aux {
  constructor(name: string) {
    super(name);
  }
  zonesFromBbox(
    depth: number,
    densify: boolean,
    bbox?: Array<Array<number>> | undefined | null
  ): any {
    const zones = super.zonesFromBbox(depth, densify, bbox);
    return decodeZones(zones);
  }
  zoneFromPoint(depth: number, point: Array<number>, densify: boolean): any {
    const zones = super.zoneFromPoint(depth, point, densify);
    return decodeZones(zones);
  }
  zonesFromParent(depth: number, parentZoneId: String, densify: boolean): any {
    const zones = super.zonesFromParent(depth, parentZoneId, densify);
    return decodeZones(zones);
  }
  zoneFromId(zoneId: string, densify: boolean): any {
    const zones = super.zoneFromId(zoneId, densify);
    return decodeZones(zones);
  }
}
