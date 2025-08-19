import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

import path from "path";
import { decodeChildren, decodeNeighbors } from "./utils";
// Needed for ESM builds to get __dirname
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const require = createRequire(import.meta.url);
// Dynamically load the addon binary
// This will resolve to dist/index.node after build

const { Dggrs: Aux } = require(path.join(__dirname, "index.node"));

export function decodeZones(zones: any) {
  const decodedZones: any = [];
  const bufferIds = Buffer.from(zones.utf8Ids);
  const bufferCoords = new Float64Array(zones.regionCoords);
  // const bufferNeighbors = new Float64Array(zones.neighborsIndex);

  for (let i = 0; i < zones.idOffsets.length; i++) {
    const start = zones.idOffsets[i];
    const end =
      i + 1 < zones.idOffsets.length
        ? zones.idOffsets[i + 1]
        : bufferIds.length;
    const id = new TextDecoder("utf-8").decode(bufferIds.subarray(start, end));

    // region coords
    const vertexCount = zones.vertexCount[i];
    const regionStart = zones.regionOffsets[i];
    const bufferRegion = bufferCoords.subarray(
      regionStart,
      regionStart + vertexCount * 2
    );
    const region = [];
    for (let j = 0; j < bufferRegion.length; j += 2) {
      region.push([bufferRegion[j], bufferRegion[j + 1]]);
    }

    // children
    const children = decodeChildren(zones, i);

    // neighbors
    const neighbors = decodeNeighbors(zones, i);

    decodedZones.push({
      id,
      center: [zones.centerX[i], zones.centerY[i]],
      vertexCount,
      region,
      children,
      neighbors,
    });
  }

  return decodedZones;
}

export class Dggrs extends Aux {
  constructor(name: string) {
    super(name);
  }
  zonesFromBbox1(
    depth: number,
    densify: boolean,
    bbox?: Array<Array<number>> | undefined | null
  ): any {
    const zones = super.zonesFromBbox1(depth, densify, bbox);
    return decodeZones(zones);
  }
}