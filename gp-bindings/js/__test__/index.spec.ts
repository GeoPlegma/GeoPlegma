import test from 'ava'

import { Dggrs } from '../dist/index'

test('sync function from native code', (t) => {
  const g = new Dggrs('isea3h')
  const rl = 3
  const bbox = [
    [-10.0, -10.0],
    [10.0, 10.0],
  ]
  let points = [
    [19.96, 5.34],
    [9.06, 52.98],
    [-29.11, -15.28],
  ]

  let ids = []
  for (const p of points) {
    const r = g.zoneFromPoint(rl, p)
    console.log(r.utf8Ids)
    // for (const id of r.utf8Ids) {
    //   ids.push(id.toString(16))
    // }
  }

  // console.log(ids)
  // const k = g.zonesFromParent(rl, '01000<0000000000000')
  // const i = g.zoneFromId('010000000000000000')
  // const a = g.zonesFromBbox(3, bbox)>
  //  const b = g.zoneFromPoint(1, [39, 9]);
  //  const c = g.zonesFromParent(1, "010000000000000000");
  //  const d = g.zoneFromId("010000000000000000");
  t.is(rl, 3)
})
