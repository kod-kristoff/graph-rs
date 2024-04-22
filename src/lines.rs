import * as R from 'ramda'
import {Diff, IndexedDiff, mass_centers, Index} from './Diff'
import * as record from '../record'
import * as Utils from '../Utils'

export type Grid<M> = Line<M>[][]

export function mapGrid<M, N>(grid: Grid<M>, f: (m: M) => N): Grid<N> {
  return grid.map(col => col.map(line => mapLine(line, f)))
}

export function mapGrids<M, N>(
  grids: {upper: Grid<M>; lower: Grid<M>},
  f: (m: M) => N
): {upper: Grid<N>; lower: Grid<N>} {
  return {
    upper: mapGrid(grids.upper, f),
    lower: mapGrid(grids.lower, f),
  }
}

export type IdGrid = Grid<{id: string}>

export function DiffToGrid(diff: Diff[]): {upper: IdGrid; lower: IdGrid} {
  return {
    upper: Grid(ProtoLines(diff, 'Dragged'), diff.length),
    lower: VFlip(Grid(ProtoLines(diff, 'Dropped'), diff.length)),
  }
}

export interface ProtoLine {
  from: number
  to: number
}

export type ProtoLines = {id: string; center_of_mass: number; lines: ProtoLine[]}[]

export function ProtoLines(diff: Diff[], keep: 'Dragged' | 'Dropped'): ProtoLines {
  const centers = mass_centers(diff)
  return R.sortBy(
    r => r.lines.length,
    record.traverse(
      R.groupBy(d => d.id, Index(diff)) as Record<string, IndexedDiff[]>,
      (ds, id) => {
        const center_of_mass = centers[ds[0].id]
        const lines = ds
          .filter(d => {
            if (d.edit == 'Edited') {
              return (
                (keep == 'Dragged' && d.source.length > 0) ||
                (keep == 'Dropped' && d.target.length > 0)
              )
            } else {
              return d.edit == keep
            }
          })
          .map(d => ({from: d.index, to: center_of_mass}))
        return {id, center_of_mass, lines}
      }
    )
  )
}

export interface Line<Meta> {
  x0: number
  x1: number
  y0: number
  y1: number
  meta: Meta
}

export function mapLine<M, N>(l: Line<M>, f: (m: M) => N): Line<N> {
  return {...l, meta: f(l.meta)}
}

export type IdLine = Line<{id: string}>

export function Grid(proto_lines: ProtoLines, width: number): IdGrid {
  const heights: number[] = new Array(width).fill(0)
  const out_lines: IdLine[][] = heights.map(_ => [] as IdLine[])
  const postponed: ((final_height: number) => void)[] = []
  proto_lines.forEach(({id, lines}) => {
    if (lines.length == 0) {
      return
    }
    const poses = Utils.flatMap(lines, pl => [pl.from, pl.to])
    const lo = Utils.minimum(poses)
    const hi = Utils.maximum(poses)
    const range = R.range(lo, hi + 1)
    const vertical = lines.length == 1 && lines[0].from == lines[0].to
    const h = Utils.maximum(range.map(i => heights[i])) + (vertical ? 0 : 1)
    range.map(i => (heights[i] = h))
    postponed.push(final_height =>
      lines.map(line => {
        if (line.from == line.to) {
          out_lines[line.from].push({
            x0: 0.5,
            y0: 0,
            x1: 0.5,
            y1: 1,
            meta: {id},
          })
        } else {
          const dir = line.to > line.from ? 'right' : 'left'
          const x0 = dir == 'left' ? 1 : 0
          const x1 = dir == 'left' ? 0 : 1
          const y = h / final_height
          out_lines[line.from].push({
            x0: 0.5,
            y0: 0,
            x1,
            y1: y,
            meta: {id},
          })
          const dx = dir == 'left' ? -1 : 1
          let x = line.from + dx
          while (x != line.to) {
            out_lines[x].push({
              x0,
              y0: y,
              x1,
              y1: y,
              meta: {id},
            })
            x += dx
          }
          out_lines[line.to].push({
            x0,
            y0: y,
            x1: 0.5,
            y1: 1,
            meta: {id},
          })
        }
      })
    )
  })
  const height = Utils.maximum(heights) + 1
  postponed.map(k => k(height))
  return out_lines
}

export function VFlip<M>(grid: Line<M>[][]): Line<M>[][] {
  return grid.map(column =>
    column.map(line => ({
      ...line,
      y0: 1 - line.y0,
      y1: 1 - line.y1,
    }))
  )
}
