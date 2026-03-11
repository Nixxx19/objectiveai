"use client";

import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";

interface ScoreOrbProps {
  score: number; // 0.0 to 1.0, drives color from red to green
  pulse?: boolean; // Whether to do a subtle pulse animation
}

const COLOR_STOPS = [
  { at: 0.0, color: new THREE.Color("#EF4444") }, // red
  { at: 0.15, color: new THREE.Color("#F97316") }, // orange
  { at: 0.33, color: new THREE.Color("#EAB308") }, // yellow
  { at: 0.66, color: new THREE.Color("#66BB6A") }, // yellow-green
  { at: 1.0, color: new THREE.Color("#22C55E") }, // green
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

export default function ScoreOrb({ score, pulse = true }: ScoreOrbProps) {
  const shellRef = useRef<THREE.Mesh>(null);
  const glowRef = useRef<THREE.Mesh>(null);
  const shellMatRef = useRef<THREE.MeshPhysicalMaterial>(null);
  const glowMatRef = useRef<THREE.MeshStandardMaterial>(null);

  const color = useMemo(() => scoreToColor(score), [score]);

  useFrame(({ clock }) => {
    if (!shellRef.current || !glowRef.current) return;

    // Update colors
    if (shellMatRef.current) {
      shellMatRef.current.color.lerp(color, 0.05);
    }
    if (glowMatRef.current) {
      glowMatRef.current.emissive.lerp(color, 0.05);
    }

    // Pulse animation
    if (pulse) {
      const t = clock.getElapsedTime();
      const scale = 1 + Math.sin(t * 1.5) * 0.03;
      shellRef.current.scale.setScalar(scale);

      // Inner glow pulses with slight offset
      const glowScale = 1 + Math.sin(t * 1.5 + 0.5) * 0.04;
      glowRef.current.scale.setScalar(glowScale);

      // Modulate emissive intensity with pulse
      if (glowMatRef.current) {
        glowMatRef.current.emissiveIntensity = 1.5 + Math.sin(t * 1.5) * 0.4;
      }
    }
  });

  return (
    <group>
      {/* Outer acrylic shell */}
      <mesh ref={shellRef}>
        <sphereGeometry args={[0.5, 64, 64]} />
        <meshPhysicalMaterial
          ref={shellMatRef}
          color={color}
          transmission={0.85}
          thickness={0.5}
          roughness={0.1}
          ior={1.45}
          clearcoat={1.0}
          clearcoatRoughness={0.05}
          transparent
          opacity={0.9}
          envMapIntensity={0.8}
        />
      </mesh>

      {/* Inner emissive glow */}
      <mesh ref={glowRef}>
        <sphereGeometry args={[0.35, 32, 32]} />
        <meshStandardMaterial
          ref={glowMatRef}
          emissive={color}
          emissiveIntensity={1.5}
          color="#000000"
          transparent
          opacity={0.6}
          toneMapped={false}
        />
      </mesh>
    </group>
  );
}
