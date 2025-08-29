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

  }

  t.is(rl, 3)
})
