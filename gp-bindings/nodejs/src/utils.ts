// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by João Manuel (GeoInsight GmbH, joao.manuel@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.

// for now the type is `any`, I will add other types in other PRs
export function decodeZones(zones: any) {
  const decodedZones: any = [];
  const bufferIds = Buffer.from(zones.utf8Ids);
  const bufferCoords = new Float64Array(zones.regionCoords);

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

export function decodeChildren(jsZones: any, zoneIndex: any) {
  const children = [];
  const start = jsZones.childrenOffsets[zoneIndex];
  const end =
    zoneIndex + 1 < jsZones.childrenOffsets.length
      ? jsZones.childrenOffsets[zoneIndex + 1]
      : jsZones.childrenIdOffsets.length;
  const buffer = new Float64Array(jsZones.childrenUtf8Ids);

  // childrenOffsets -> [0, 6, 12,...]
  // childrenIdOffsets -> [0, 18, 36,...]
  for (let i = start; i < end; i++) {
    const childStart = jsZones.childrenIdOffsets[i];
    const childEnd =
      i + 1 < jsZones.childrenIdOffsets.length
        ? jsZones.childrenIdOffsets[i + 1]
        : jsZones.childrenUtf8Ids.length;
    children.push(
      new TextDecoder("utf-8").decode(buffer.subarray(childStart, childEnd))
    );
  }

  return children;
}

export function decodeNeighbors(jsZones: any, zoneIndex: any) {
  const neighbors = [];
  const start = jsZones.neighborsOffsets[zoneIndex];
  const end =
    zoneIndex + 1 < jsZones.neighborsOffsets.length
      ? jsZones.neighborsOffsets[zoneIndex + 1]
      : jsZones.neighborsIdOffsets.length;

  const buffer = new Float64Array(jsZones.neighborsUtf8Ids);
  for (let i = start; i < end; i++) {
    const nStart = jsZones.neighborsIdOffsets[i];
    const nEnd =
      i + 1 < jsZones.neighborsIdOffsets.length
        ? jsZones.neighborsIdOffsets[i + 1]
        : jsZones.neighborsUtf8Ids.length;
    neighbors.push(
      new TextDecoder("utf-8").decode(buffer.subarray(nStart, nEnd))
    );
  }
  return neighbors;
}
