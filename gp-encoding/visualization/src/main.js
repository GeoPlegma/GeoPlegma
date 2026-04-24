import {Deck} from '@deck.gl/core';
import {H3HexagonLayer} from '@deck.gl/geo-layers';
import {PolygonLayer} from '@deck.gl/layers';

const presets = {
  'large_raster': {
    lon:  [4.326247, 4.469885],
    lat:  [52.166899, 52.258604],
    elevation: 0,
    fillColor: d => [
      Math.round((d.band_2 / 11652) * 255) * 7,
      Math.round((d.band_1 / 11652) * 255) * 7,
      Math.round((d.band_0 / 11652) * 255) * 7
    ]
  },
  'elevation': {
    lon: [-9.238194, -9.086250],
    lat: [38.679861, 38.796806],
    elevation: d => d.band_0 / 2,
    fillColor: d => [d.band_0, d.band_0, d.band_0],
  }
}

const {lon, lat, elevation, fillColor} = presets['elevation'];

const BBOX_POLYGON = [
  [lon[0], lat[0]],
  [lon[1], lat[0]],
  [lon[1], lat[1]],
  [lon[0], lat[1]],
  [lon[0], lat[0]]
];


const layer = new H3HexagonLayer({
  id: 'H3HexagonLayer',
  data: '/h3cells.json',
  elevationScale: 20,
  extruded: true,
  filled: true,
  getElevation: elevation,
  getFillColor: fillColor,
  getHexagon: d => d.hex,
  wireframe: false,
  pickable: true,
});

const boundingBoxLayer = new PolygonLayer({
  id: 'BoundingBoxLayer',
  data: [{polygon: BBOX_POLYGON}],
  getPolygon: d => d.polygon,
  stroked: true,
  filled: true,
  lineWidthMinPixels: 3,
  getLineColor: [230, 57, 70, 255],
  getFillColor: [230, 57, 70, 35],
  getLineWidth: 2,
  pickable: true
});

const appContainer = document.getElementById('app');

if (appContainer) {
  appContainer.addEventListener('contextmenu', event => {
    event.preventDefault();
  });
}

new Deck({
  parent: appContainer || document.body,
  mapStyle: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
  initialViewState: {
    longitude: (lon[0] + lon[1]) / 2,
    latitude: (lat[0] + lat[1]) / 2,
    zoom: 11,
    maxZoom: 20,
    pitch: 30,
    bearing: 0
  },
  controller: true,
  getTooltip: ({object}) => object && `${object.hex} band_0: ${object.band_0}`,
  layers: [
    layer, 
    // boundingBoxLayer
  ]
});
  