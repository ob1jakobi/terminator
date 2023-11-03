-- Users Table
INSERT INTO Users (Username, Password) VALUES ("Jake", "PASSWORD");

-- Exams Table
INSERT INTO Exams (Title, Description) VALUES ("EXAM1", "Test Exam...");

-- ExamCreations Table
INSERT INTO ExamCreation (ExamID, CreatorUsername, DateCreated) VALUES (1, "Jake", "03NOV23");

INSERT INTO Questions (QuestionText, Options, CorrectAnswer, Explanation, ExamID) VALUES
    ("Test question prompt:", "A.) 1&B.) 2&C.) 3", "B.) 2", "Because I said so...", 1);