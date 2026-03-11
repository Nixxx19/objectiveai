"use client";

import { useRef, useMemo, MutableRefObject, createRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Environment } from "@react-three/drei";
import * as THREE from "three";

import ThinkerFigure from "./ThinkerFigure";
import ScoreOrb from "./ScoreOrb";

interface ThinkerCouncilProps {
  scrollProgressRef: MutableRefObject<number>;
  reducedMotion: boolean;
}

// Figure layout: 5 figures in a semi-circle, sizes represent weights
const FIGURES = [
  { angle: -20, scale: 1.0, nodDelay: 0 },     // center-left (large)
  { angle: -70, scale: 0.75, nodDelay: 0.15 },  // far-left (medium)
  { angle: 25, scale: 0.85, nodDelay: 0.08 },   // center-right (medium)
  { angle: 75, scale: 0.7, nodDelay: 0.22 },    // far-right (small)
  { angle: 0, scale: 0.65, nodDelay: 0.3 },     // back-center (small)
];

const SEMICIRCLE_RADIUS = 4;

// Score color interpolation
const COLOR_STOPS = [
  { at: 0.0, color: new THREE.Color("#EF4444") },
  { at: 0.15, color: new THREE.Color("#F97316") },
  { at: 0.33, color: new THREE.Color("#EAB308") },
  { at: 0.66, color: new THREE.Color("#66BB6A") },
  { at: 1.0, color: new THREE.Color("#22C55E") },
];

function scoreToColor(score: number): THREE.Color {
  const s = THREE.MathUtils.clamp(score, 0, 1);
  for (let i = 0; i < COLOR_STOPS.length - 1; i++) {
    const curr = COLOR_STOPS[i];
    const next = COLOR_STOPS[i + 1];
    if (s >= curr.at && s <= next.at) {
      const t = (s - curr.at) / (next.at - curr.at);
      return new THREE.Color().lerpColors(curr.color, next.color, t);
    }
  }
  return COLOR_STOPS[COLOR_STOPS.length - 1].color.clone();
}

function scrollToState(progress: number) {
  let score: number;
  let deliberation: number;
  let voting: number;

  if (progress < 0.2) {
    score = 0.3;
    deliberation = 0;
    voting = 0;
  } else if (progress < 0.4) {
    const t = (progress - 0.2) / 0.2;
    score = 0.3;
    deliberation = t * 0.3;
    voting = 0;
  } else if (progress < 0.6) {
    const t = (progress - 0.4) / 0.2;
    score = 0.3 + t * 0.2;
    deliberation = 0.3 + t * 0.7;
    voting = 0;
  } else if (progress < 0.8) {
    const t = (progress - 0.6) / 0.2;
    score = 0.5 + t * 0.3;
    deliberation = 1.0;
    voting = t;
  } else {
    const t = (progress - 0.8) / 0.2;
    score = 0.8 + t * 0.2;
    deliberation = 1.0 - t * 0.5;
    voting = 1.0;
  }

  return { score, deliberation, voting };
}

export default function ThinkerCouncil({ scrollProgressRef, reducedMotion }: ThinkerCouncilProps) {
  const pointLightRef = useRef<THREE.PointLight>(null);
  const orbScoreRef = useRef(reducedMotion ? 1.0 : 0.3);

  // Refs for each figure's head group (for per-frame animation)
  const headRefs = useMemo(
    () => FIGURES.map(() => createRef<THREE.Group>()),
    []
  );

  // Mutable animation state (no React re-renders)
  const anim = useRef({
    score: reducedMotion ? 1.0 : 0.3,
    deliberation: 0,
    voting: reducedMotion ? 1.0 : 0,
    headTilts: FIGURES.map(() => 0),
    headNods: FIGURES.map(() => 0),
  });

  // Pre-compute figure positions (static)
  const figurePositions = useMemo(() => {
    return FIGURES.map((fig, i) => {
      const angleRad = THREE.MathUtils.degToRad(fig.angle - 90);
      const radius = i === 4 ? SEMICIRCLE_RADIUS + 1.5 : SEMICIRCLE_RADIUS;
      const x = Math.cos(angleRad) * radius;
      const z = Math.sin(angleRad) * radius;
      const rotationY = Math.atan2(-x, -z);

      return {
        position: [x, 0, z] as [number, number, number],
        rotation: [0, rotationY, 0] as [number, number, number],
        scale: fig.scale,
      };
    });
  }, []);

  useFrame(({ clock }) => {
    const a = anim.current;
    const progress = scrollProgressRef.current;
    const target = reducedMotion
      ? { score: 1.0, deliberation: 0, voting: 1.0 }
      : scrollToState(progress);

    // Smooth interpolation
    a.score = THREE.MathUtils.lerp(a.score, target.score, 0.08);
    a.deliberation = THREE.MathUtils.lerp(a.deliberation, target.deliberation, 0.06);
    a.voting = THREE.MathUtils.lerp(a.voting, target.voting, 0.06);

    const t = clock.getElapsedTime();

    // Update score for orb (ScoreOrb reads this via its own useFrame)
    orbScoreRef.current = a.score;

    // Per-figure head animation via refs
    for (let i = 0; i < FIGURES.length; i++) {
      const fig = FIGURES[i];
      const headGroup = headRefs[i].current;
      if (!headGroup) continue;

      // Organic idle sway
      const idleTilt = Math.sin(t * 0.5 + i * 1.2) * 0.03;
      // Deliberation: staggered tilts
      const deliberationTilt = Math.sin(t * 0.8 + fig.nodDelay * 10) * 0.12 * a.deliberation;
      // Voting nod: forward nod, staggered by delay
      const nodProgress = THREE.MathUtils.clamp(
        (a.voting - fig.nodDelay) / (1 - fig.nodDelay), 0, 1
      );
      const votingNod = -0.25 * nodProgress;

      // Smooth current values
      a.headTilts[i] = THREE.MathUtils.lerp(a.headTilts[i], idleTilt + deliberationTilt, 0.1);
      a.headNods[i] = THREE.MathUtils.lerp(a.headNods[i], votingNod, 0.08);

      // Apply to head group rotation
      headGroup.rotation.set(a.headNods[i], a.headTilts[i], 0);
    }

    // Update point light color
    if (pointLightRef.current) {
      const color = scoreToColor(a.score);
      pointLightRef.current.color.copy(color);
    }
  });

  return (
    <>
      {/* Environment for reflections */}
      <Environment preset="city" environmentIntensity={0.3} />

      {/* Fog for depth fade */}
      <fog attach="fog" args={["#1B1B1B", 6, 16]} />

      {/* Lighting */}
      <ambientLight intensity={0.4} />
      <directionalLight position={[4, 6, 2]} intensity={0.8} />

      {/* Point light near the orb */}
      <pointLight
        ref={pointLightRef}
        position={[0, 0.5, 0]}
        intensity={2.5}
        distance={8}
        decay={2}
      />

      {/* Central Score Orb — reads orbScoreRef per frame internally */}
      <ScoreOrb score={orbScoreRef.current} pulse />

      {/* Thinker Figures — head rotation driven by refs in useFrame */}
      {figurePositions.map((fig, i) => (
        <ThinkerFigure
          key={i}
          ref={headRefs[i]}
          position={fig.position}
          rotation={fig.rotation}
          scale={fig.scale}
          headTilt={0}
          headNod={0}
        />
      ))}
    </>
  );
}
