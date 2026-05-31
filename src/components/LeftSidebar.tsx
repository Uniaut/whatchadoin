import type { Tab } from "../app-view/App";

interface Props {
  tab: Tab;
  onTabChange: (tab: Tab) => void;
  dates: string[];
  selectedDate: string;
  onSelectDate: (date: string) => void;
}

function LeftSidebar({
  tab,
  onTabChange,
  dates,
  selectedDate,
  onSelectDate,
}: Props) {
  return (
    <aside className="left-sidebar">
      <nav className="tab-switcher">
        <button
          className={`tab-btn ${tab === "work" ? "active" : ""}`}
          onClick={() => onTabChange("work")}
        >
          Work
        </button>
        <button
          className={`tab-btn ${tab === "report" ? "active" : ""}`}
          onClick={() => onTabChange("report")}
        >
          Report
        </button>
      </nav>

      {tab === "work" ? (
        <section className="left-sidebar-body">
          <span className="placeholder-text">placeholder</span>
        </section>
      ) : (
        <ul className="date-list">
          {dates.map((date) => (
            <li key={date}>
              <button
                className={`date-item ${
                  date === selectedDate ? "active" : ""
                }`}
                onClick={() => onSelectDate(date)}
              >
                {date}
              </button>
            </li>
          ))}
        </ul>
      )}
    </aside>
  );
}

export default LeftSidebar;
