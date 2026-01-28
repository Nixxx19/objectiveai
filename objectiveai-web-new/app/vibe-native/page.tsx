export default function VibeNativePage() {
  return (
    <div className="page" style={{
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '50vh',
      padding: '60px 32px',
      textAlign: 'center',
    }}>
      <span className="tag" style={{ marginBottom: '16px' }}>
        Coming Soon
      </span>
      <h1 style={{
        fontSize: '32px',
        fontWeight: 700,
        marginBottom: '12px',
      }}>
        Vibe-Native
      </h1>
      <p style={{ color: 'var(--text-muted)' }}>
        Use ObjectiveAI functions through natural language—no code required.
      </p>
    </div>
  );
}
