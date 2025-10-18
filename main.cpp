#include <iostream>
#include <vector>
#include <set>
#include <algorithm>
#include <cmath>
#include <queue>
#include <functional>

// using namespace std; を使うことで、std:: を省略できます。
using namespace std;

// === グローバル変数の宣言 ===
// Pythonのグローバル変数に対応します。
const int N = 81; // グリッドのサイズ (9x9)
vector<vector<int>> G; // グラフの隣接リスト
vector<int> mother;      // DFS木における親ノード
vector<int> lowlink;     // Tarjan's algorithm で使用
vector<int> ord;         // DFSでの訪問順
vector<int> check;       // 訪問済みフラグ (0: 未訪問, 1: 訪問中/済み)
int counter;             // ord のタイムスタンプ
int cnt;                 // 根ノード (ノード0) の子の数
set<int> blocked;        // ブロックされたマスの座標を格納
set<int> initial_obstacles;

/**
 * @brief 関節点を見つけるための深さ優先探索 (DFS)
 * @param v 現在のノード
 */
void dfs(int v) {
    // 根ノードの場合、子の数を初期化
    if (v == 0) {
        cnt = 0;
    }
    ord[v] = counter;
    lowlink[v] = ord[v];
    counter++;
    check[v] = 1;

    for (int i : G[v]) {
        if (check[i] == 0) { // 未訪問の隣接ノード
            mother[i] = v;
            dfs(i);
            lowlink[v] = min(lowlink[v], lowlink[i]);
            if (v == 0) {
                cnt++;
            }
        } else if (i != mother[v]) { // 後退辺の場合
            lowlink[v] = min(lowlink[v], ord[i]);
        }
    }
}

/**
 * @brief 関節点ではないノード（配置可能なマス）のリストを返す
 * @return 配置可能なマスの座標リスト
 */
vector<int> find_non_articulation_points() {
    // 各種ベクターとカウンターを初期化
    ord.assign(N, -1);
    mother.assign(N, -1);
    lowlink.assign(N, -1);
    check.assign(N, 0);
    counter = 0;
    cnt = 0;

    dfs(0);

    // 関節点を特定
    vector<int> art_vec;
    for (int i = 0; i < N; i++) {
        if (i == 0 || mother[i] == -1) continue;
        // 根ノードではなく、ord[parent] <= lowlink[child] を満たす場合に親が関節点
        if (ord[mother[i]] <= lowlink[i] && mother[i] != 0) {
            art_vec.push_back(mother[i]);
        }
    }
    // 根ノードは子の数が2以上の場合に関節点
    if (cnt > 1) {
        art_vec.push_back(0);
    }

    set<int> articulation_points(art_vec.begin(), art_vec.end());

    // 関節点でもなく、ブロックされてもいないマスをリストアップ
    vector<int> result;
    for (int i = 0; i < N; i++) {
        // ノード番号iから元のグリッド座標pへの変換は p = (i + 4) % N
        int p = (i + 4) % N;
        if (i == 0 || blocked.count(p)) { // 中心(ノード0)やブロックされたマスは除外
            continue;
        }
        if (articulation_points.find(i) == articulation_points.end()) {
            result.push_back(p);
        }
    }
    return result;
}


int main() {
    // 高速入出力
    ios_base::sync_with_stdio(false);
    cin.tie(NULL);

    int D, initial_blocks;
    cin >> D >> initial_blocks;

    //各マスについて、「周りに空きマスがいくつあるか」を管理する。
    vector<int> empty_count(D*D, 0);
    //すでに登場したブロックの番号を管理する。
    vector<bool> appeared(D*D, false);
    // 初期化: empty_count を blocked に基づいて作る
    for (int r = 0; r < D; ++r) {
        for (int c = 0; c < D; ++c) {
            int idx = r * D + c;
            int count = 0;
            if (r > 0 && blocked.count((r - 1) * D + c) == 0) count++; // 上
            if (r < D - 1 && blocked.count((r + 1) * D + c) == 0) count++; // 下
            if (c > 0 && blocked.count(r * D + (c - 1)) == 0) count++; // 左
            if (c < D - 1 && blocked.count(r * D + (c + 1)) == 0) count++; // 右
            empty_count[idx] = count;
        }
    }
    // 各マスについて、「周囲のマスについて一番小さいのは何か」を管理する。
    vector<int> min_containar(D*D, 0);
    // 各マスについて、「自分より小さいのがあるか」を管理する。
    vector<bool> has_smaller(D*D, false);

    for (int i = 0; i < initial_blocks; ++i) {
        int r, c;
        cin >> r >> c;
        blocked.insert(r * D + c);
        initial_obstacles.insert(r * D + c);
    }

    vector<int> cntn(N, -1);
    int num_to_place = D * D - 1 - initial_blocks;

    //　入り口から奥までの最短経路を事前計算しておく
    vector<vector<int>> dist_from_entrance(D, vector<int>(D, -1));
    queue<pair<int, int>> q;
    int entrance = 4; // (0,4) の位置
    dist_from_entrance[entrance/D][entrance%D] = 0;
    q.push({entrance/D, entrance%D});
    while (!q.empty()) {
        auto [r, c] = q.front();
        q.pop();
        int d = dist_from_entrance[r][c];
        int directions[4][2] = {{-1,0},{1,0},{0,-1},{0,1}};
        for (auto& dir : directions) {
            int nr = r + dir[0];
            int nc = c + dir[1];
            if (nr >= 0 && nr < D && nc >= 0 && nc < D) {
                int np = nr * D + nc;
                if (dist_from_entrance[nr][nc] == -1 && blocked.count(np) == 0) {
                    dist_from_entrance[nr][nc] = d + 1;
                    q.push({nr, nc});
                }
            }
        }
    }

    // dist_from_entrance[D-1]の最小値に対応する地点への最短経路を復元する
    vector<vector<bool>> sanctuary(D, vector<bool>(D, false));
    int min_dist = 1e9;
    int nearest_c = -1;
    for (int c = 0; c < D; ++c) {
        if (dist_from_entrance[D-1][c] != -1 && dist_from_entrance[D-1][c] < min_dist) {
            min_dist = dist_from_entrance[D-1][c];
            nearest_c = c;
        }
    }
    int r = D - 1 , c = nearest_c;
    while (r != 0 || c != 4) {
        sanctuary[r][c] = true;
        int directions[4][2] = {{-1,0},{1,0},{0,-1},{0,1}};
        for (auto& dir : directions) {
            int nr = r + dir[0];
            int nc = c + dir[1];
            if (nr >= 0 && nr < D && nc >= 0 && nc < D) {
                if (dist_from_entrance[nr][nc] == dist_from_entrance[r][c] - 1) {
                    r = nr;
                    c = nc;
                    break;
                }
            }
        }
    }
    sanctuary[0][4] = true; //入り口も聖域に含める



    // === コンテナ配置フェーズ ===
    for (int k = 0; k < num_to_place; ++k) {
        G.assign(N, vector<int>());
        // グラフ構築
        for (int r = 0; r < D; ++r) {
            for (int c = 0; c < D; ++c) {
                int s = r * D + c;
                int neighbors[] = {s + 1, s + D};
                bool conditions[] = {c < D - 1, r < D - 1};
                for(int j = 0; j < 2; ++j) {
                    if (conditions[j]) {
                        int t = neighbors[j];
                        if (!blocked.count(s) && !blocked.count(t)) {
                            // グリッド座標pからノード番号vへの変換: v = (p - 4 + N) % N
                            int u = (s - 4 + N) % N;
                            int v_node = (t - 4 + N) % N;
                            G[u].push_back(v_node);
                            G[v_node].push_back(u);
                        }
                    }
                }
            }
        }
        
        int td;
        cin >> td;

        vector<int> candidates = find_non_articulation_points();

        if (candidates.empty()) { // 候補がない場合のフォールバック
            for (int i = 0; i < N; ++i) {
                if (!blocked.count(i) && i != 4) {
                    candidates.push_back(i);
                    break;
                }
            }
        }

        // 聖域の外のマスについて、rが小さい順に番号を付ける。
        // 各rについて、聖域以外の空きマスを数える
        vector<int> empty_place_counts(D + 1,0);
        empty_place_counts[D] = 1e9; // 番兵
        for (int r = 0; r < D; ++r) {
            int count = 0;
                for (int c = 0; c < D; ++c) {
                int p = r * D + c;
                if (sanctuary[r][c] == false && blocked.count(p) == 0) {
                    count++;
                }
            }
            empty_place_counts[r] = count;
        }

        vector<int> sanctuary_free_moderate(D*D, 0);
        int r = 0;
        for (int i = 0; i < D*D; ++i){
            if (appeared[i]) {
                sanctuary_free_moderate[i] = D; // 番兵
                continue;
            }
            while (empty_place_counts[r] == 0){
                r++;
            }
            sanctuary_free_moderate[i] = r;
            empty_place_counts[r]--;
        }
        // //sanctuary_free_moderateを出力
        // cerr<<"sanctuary_free_moderate: ";
        // for (int i = 0; i < D*D; ++i){
        //     cerr<<sanctuary_free_moderate[i]<<" ";
        // }

        // 最適な配置場所を選択
        int pt = candidates[0];
        int evaluation = -1e9;
        for (int c : candidates) {
            int eval = 0;
            // 評価を計算する。
            eval -= abs(sanctuary_free_moderate[td] - c/D) * 100;
            if (sanctuary[c/D][c%D]) {
                eval -= 10000; // 聖域内は大幅にペナルティ
            }
            if (empty_count[c] < 4) {
                eval += 30 - empty_count[c] * 5;
            }
            if (eval > evaluation) {
                if (td > 70){
                    cerr << td <<  " "<< "Candidate: " << c << ", Eval: " << eval << endl;
                }
                evaluation = eval;
                pt = c;
            }
        }

        // 周囲の empty_count を更新
        int c;
        r = pt / D;c = pt % D;
        if (r > 0) empty_count[(r - 1) * D + c]--;
        if (r < D - 1) empty_count[(r + 1) * D + c]--;
        if (c > 0) empty_count[r * D + (c - 1)]--;
        if (c < D - 1) empty_count[r * D + (c + 1)]--;

        cout << pt / D << " " << pt % D << endl;
        cntn[pt] = td;
        appeared[td] = true;
        blocked.insert(pt);
    }

    // === コンテナ運び出しフェーズ (ビームサーチ版) ===

    // ビームサーチの幅: この値を大きくすると探索の幅が広がるが、計算時間がかかる
    const int BEAM_WIDTH = 1000;

    // ビームサーチの状態を管理する構造体
    struct State {
        long long score;          // 評価値 (これまでの転倒数)
        vector<int> path;         // 運び出し経路 (コンテナの座標リスト)
        vector<bool> unejected;   // まだ運び出されていないコンテナのフラグ

        // 評価値が小さい順にソートするための比較演算子
        bool operator<(const State& other) const {
            return score < other.score;
        }
    };

    // 1. 初期状態の作成
    State initial_state;
    initial_state.score = 0;
    initial_state.unejected.assign(N, false);
    for(int i = 0; i < N; ++i) {
        if (cntn[i] != -1) {
            initial_state.unejected[i] = true; // 配置されたコンテナをセット
        }
    }
    initial_state.unejected[4] = false; // 中心は最初から空いている

    // 2. ビームの初期化
    vector<State> beam;
    beam.push_back(initial_state);

// 3. 運び出すコンテナの数だけ探索を繰り返す
for (int i = 0; i < num_to_place; ++i) {
    vector<State> candidates;

    for (const auto& current_state : beam) {
        vector<int> ejectable_now;
        
        if (i == 0) {
            // ループの初回は、中央(4)に隣接するものだけが候補
            int initial_candidates[] = {3, 5, 13}; // (0,3), (0,5), (1,4)
            for (int p : initial_candidates) {
                if (current_state.unejected[p]) {
                    ejectable_now.push_back(p);
                }
            }
        } else {
            // 2手目以降は、すでに空いたマスに隣接するものを探す
            for (int p = 0; p < N; ++p) {
                if (current_state.unejected[p]) {
                    int r = p / D, c = p % D;
                    bool can_eject = false;
                // 隣接マスが「空いている」かつ「初期障害物ではない」ことを確認
                if (r > 0 && !current_state.unejected[p - D] && initial_obstacles.count(p - D) == 0) can_eject = true;
                if (r < D - 1 && !current_state.unejected[p + D] && initial_obstacles.count(p + D) == 0) can_eject = true;
                if (c > 0 && !current_state.unejected[p - 1] && initial_obstacles.count(p - 1) == 0) can_eject = true;
                if (c < D - 1 && !current_state.unejected[p + 1] && initial_obstacles.count(p + 1) == 0) can_eject = true;
                    
                    if (can_eject) {
                        ejectable_now.push_back(p);
                    }
                }
            }
        }
        
        // 各運び出し可能な手に対して、新しい状態を生成
        for (int p_to_eject : ejectable_now) {
            State next_state = current_state;
            next_state.path.push_back(p_to_eject);
            next_state.unejected[p_to_eject] = false;

            next_state.score += (i + 1) * cntn[p_to_eject] + current_state.score;

            candidates.push_back(next_state);
        }
    }

        // 4. 候補の中からスコアの良いものをビーム幅だけ残す (Pruning)
        sort(candidates.begin(), candidates.end());
        if (candidates.size() > BEAM_WIDTH) {
            candidates.resize(BEAM_WIDTH);
        }
        beam = candidates;
        
        // 候補がなければ探索終了
        if (beam.empty()) break;
    }

    // 5. 最終的に残ったビームの中で最もスコアの良いものを選ぶ
    if (!beam.empty()) {
        const auto& best_state = *min_element(beam.begin(), beam.end());
        for (int p : best_state.path) {
            cout << p / D << " " << p % D << endl;
        }
    }


    return 0;
}