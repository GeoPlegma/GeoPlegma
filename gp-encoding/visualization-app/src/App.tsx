import { useState, useMemo } from 'react';
import DeckGL from '@deck.gl/react';
import { _GlobeView as GlobeView, WebMercatorViewport } from '@deck.gl/core';
import { H3HexagonLayer } from '@deck.gl/geo-layers';
import { GeoJsonLayer } from '@deck.gl/layers';
import { cellToBoundary } from 'h3-js';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

const BAND_KEY_PREFIX = 'band_';

function getBandCount(sample: any) {
  if (!sample || typeof sample !== 'object') {
    return 0;
  }
  return Object.keys(sample).filter(key => key.startsWith(BAND_KEY_PREFIX)).length;
}

const COUNTRIES_BORDERS_URL = '/countries.geojson';
const VIEW_PADDING = 64;

function getBoundsFromH3Cells(cells: any[]) {
  const bounds = {
    west: Number.POSITIVE_INFINITY,
    south: Number.POSITIVE_INFINITY,
    east: Number.NEGATIVE_INFINITY,
    north: Number.NEGATIVE_INFINITY
  };

  for (const cell of cells) {
    const boundary = cellToBoundary(cell.hex, true);

    for (const [longitude, latitude] of boundary) {
      bounds.west = Math.min(bounds.west, longitude);
      bounds.south = Math.min(bounds.south, latitude);
      bounds.east = Math.max(bounds.east, longitude);
      bounds.north = Math.max(bounds.north, latitude);
    }
  }

  if (!Number.isFinite(bounds.west)) {
    return [[-180, -90], [180, 90]];
  }

  return [
    [bounds.west, bounds.south],
    [bounds.east, bounds.north]
  ];
}

function getInitialViewState(bounds: number[][]) {
  const width = window.innerWidth;
  const height = window.innerHeight;
  const viewport = new WebMercatorViewport({ width, height });
  const { longitude, latitude, zoom } = viewport.fitBounds(bounds as any, {
    padding: VIEW_PADDING,
    maxZoom: 20
  });

  return {
    longitude,
    latitude,
    zoom,
    maxZoom: 20,
    pitch: 30,
    bearing: 0
  };
}

const earthMaskLayer = new GeoJsonLayer({
  id: 'EarthMaskLayer',
  data: {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        properties: {},
        geometry: {
          type: 'Polygon',
          coordinates: [[
            [-180, -90],
            [-180, 90],
            [180, 90],
            [180, -90],
            [-180, -90]
          ]]
        }
      }
    ]
  },
  stroked: false,
  filled: true,
  getFillColor: [255, 255, 255, 255],
  pickable: false
});

const countriesBordersLayer = new GeoJsonLayer({
  id: 'CountriesBordersLayer',
  data: COUNTRIES_BORDERS_URL,
  stroked: true,
  filled: false,
  lineWidthMinPixels: 1,
  getLineColor: [120, 132, 145, 120],
  pickable: false
});

function App() {
  const [storePath, setStorePath] = useState('./tmp/gp_encoding_geotiff_convert');
  const [level, setLevel] = useState(2);
  const [cells, setCells] = useState<any[]>([]);
  const [viewState, setViewState] = useState({
    longitude: 0,
    latitude: 0,
    zoom: 1,
    maxZoom: 20,
    pitch: 30,
    bearing: 0
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      const data: any[] = await invoke('get_h3_data', { store: storePath, level: Number(level) });
      setCells(data);
      if (data.length > 0) {
        const bounds = getBoundsFromH3Cells(data);
        setViewState(getInitialViewState(bounds));
      }
    } catch (err: any) {
      console.error(err);
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const layers = useMemo(() => {
    const layerList: any[] = [countriesBordersLayer, earthMaskLayer];

    if (cells.length > 0) {
      const bandCount = getBandCount(cells[0]);
      const useElevation = bandCount === 1;

      const h3Layer = new H3HexagonLayer({
        id: 'H3HexagonLayer',
        data: cells,
        elevationScale: 20,
        extruded: useElevation,
        filled: true,
        getElevation: useElevation ? (d: any) => d.band_0 / 2 : 0,
        getFillColor: useElevation
          ? (d: any) => [d.band_0, d.band_0, d.band_0]
          : (d: any) => [d.band_0, d.band_1, d.band_2],
        getHexagon: (d: any) => d.hex,
        wireframe: false,
        pickable: true,
      });

      layerList.push(h3Layer);
    }

    return layerList;
  }, [cells]);

  return (
    <div style={{ width: '100vw', height: '100vh', position: 'relative' }} onContextMenu={(e) => e.preventDefault()}>
      <DeckGL
        views={new GlobeView({ id: 'globe' })}
        initialViewState={viewState}
        controller={true}
        layers={layers}
        getTooltip={({ object }: any) => object && `${object.hex}`}
        style={{ backgroundColor: '#000' }}
      />
      
      <div className="control-panel">
        <h3>GeoPlegma Store Loader</h3>
        <div className="input-group">
          <label>Store Path</label>
          <input 
            type="text" 
            value={storePath} 
            onChange={e => setStorePath(e.target.value)} 
          />
        </div>
        <div className="input-group">
          <label>Level</label>
          <input 
            type="number" 
            value={level} 
            onChange={e => setLevel(parseInt(e.target.value) || 0)} 
          />
        </div>
        <button onClick={loadData} disabled={loading}>
          {loading ? 'Loading...' : 'Load Data'}
        </button>
        {error && <div className="error">{error}</div>}
      </div>
    </div>
  );
}

export default App;
