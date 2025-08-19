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