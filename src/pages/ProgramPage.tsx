import { useParams } from "react-router";
import { useReceiver } from "../lib/useReceiver";
import { ProgramStage } from "../components/ProgramStage";

export function ProgramPage() {
  const { sessionId } = useParams();
  const receiver = useReceiver(sessionId, "program");

  if (!sessionId) {
    return (
      <main className="program-page">
        <h1>Invalid program URL</h1>
      </main>
    );
  }

  return (
    <main className="program-page">
      <ProgramStage streams={receiver.streams} pgm={receiver.pgm} orientations={receiver.orientations} />
      {receiver.error ? <p className="error overlay-msg">{receiver.error}</p> : null}
    </main>
  );
}
