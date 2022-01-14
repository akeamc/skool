import { DateTime } from "luxon";
import { FunctionComponent, memo } from "react";
import styles from "./scale.module.scss";

const ScaleLabel: FunctionComponent<{hour: number}> = ({hour}) => (
  <div className={styles.label} style={{["--hour" as any]: hour}}>
    <time>
      {hour.toString().padStart(2, "0")}:00
    </time>
    <hr className={styles.full} style={{top: 0}} />
    <hr className={styles.quarter} style={{top: "25%"}} />
    <hr className={styles.half} style={{top: "50%"}} />
    <hr className={styles.quarter} style={{top: "75%"}} />
    </div>
  );
  
  const UncachedScale: FunctionComponent = () => {
    return <div className={styles.scale}>
      {Array.from({length: 24}).map((_, i) => <ScaleLabel hour={i} key={i} />)}
    </div>;
  };
  
export const Scale = memo(UncachedScale);
